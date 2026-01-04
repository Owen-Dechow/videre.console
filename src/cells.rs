use std::fmt::Debug;

use serde_json::{Map, Value};
use unicode_width::UnicodeWidthStr;

use crate::{connections::LayerConnector, value::VidereValue};

#[derive(Debug)]
pub enum VidereCell {
    Obj(Vec<(String, VidereValue)>),
    Arr(Vec<VidereValue>),
}

impl VidereCell {
    const WALLS_WIDTH: usize = 2;

    const TOP_LEFT: &str = "┬";
    const ORIGIN_TOP_LEFT: &str = "╭";
    const TOP_RIGHT: &str = "╮";
    const BOTTOM_LEFT: &str = "╰";
    const BOTTOM_RIGHT: &str = "╯";
    const HORIZONTAL_EDGE: &str = "─";
    const VERTICAL_EDGE: &str = "│";
    const CROSS_UP: &str = "┴";
    const CROSS_DOWN: &str = "┬";
    const CROSS_RIGHT: &str = "├";

    fn get_min_width(&self) -> usize {
        Self::WALLS_WIDTH
            + match self {
                VidereCell::Obj(values) => {
                    values.iter().map(|(k, _)| k.width()).max().unwrap_or(0)
                        + values
                            .iter()
                            .map(|(_, v)| v.get_min_width())
                            .max()
                            .unwrap_or(0)
                        + 1
                }
                VidereCell::Arr(values) => {
                    values.iter().map(|v| v.get_min_width()).max().unwrap_or(0)
                }
            }
    }

    fn get_height(&self) -> usize {
        Self::WALLS_WIDTH
            + match self {
                VidereCell::Obj(values) => values.len(),
                VidereCell::Arr(values) => values.len(),
            }
    }

    fn get_top_row(width: usize, cross: Option<usize>, layer: usize) -> String {
        let mut row = String::from(match layer == 0 {
            true => Self::ORIGIN_TOP_LEFT,
            false => Self::TOP_LEFT,
        });

        match cross {
            Some(cross) => {
                row.push_str(&Self::HORIZONTAL_EDGE.repeat(cross));
                row.push_str(Self::CROSS_DOWN);
                row.push_str(&Self::HORIZONTAL_EDGE.repeat(width - cross - 3));
            }
            None => row.push_str(&Self::HORIZONTAL_EDGE.repeat(width - 2)),
        };
        row.push_str(Self::TOP_RIGHT);
        return row;
    }

    fn get_bottom_row(width: usize, cross: Option<usize>) -> String {
        let mut row = String::from(Self::BOTTOM_LEFT);
        match cross {
            Some(cross) => {
                row.push_str(&Self::HORIZONTAL_EDGE.repeat(cross));
                row.push_str(Self::CROSS_UP);
                row.push_str(&Self::HORIZONTAL_EDGE.repeat(width - cross - 3));
            }
            None => row.push_str(&Self::HORIZONTAL_EDGE.repeat(width - 2)),
        };
        row.push_str(Self::BOTTOM_RIGHT);
        return row;
    }

    fn get_rows(
        &self,
        width: usize,
        from_offset: usize,
        layer: usize,
    ) -> (Vec<String>, Vec<usize>) {
        let mut rows = Vec::new();
        let mut from = Vec::new();

        match self {
            VidereCell::Obj(values) => {
                let min_key_width = values.iter().map(|k| k.0.width()).max().unwrap_or(0);
                rows.push(Self::get_top_row(width, Some(min_key_width), layer));

                for (key, val) in values {
                    let right_edge = if let VidereValue::Object | VidereValue::Array = val {
                        from.push(rows.len() + from_offset);
                        Self::CROSS_RIGHT
                    } else {
                        Self::VERTICAL_EDGE
                    };

                    let mut row = String::from(Self::VERTICAL_EDGE);
                    row.push_str(&key);
                    row.push_str(&" ".repeat(min_key_width - key.width()));
                    row.push_str(Self::VERTICAL_EDGE);
                    row.push_str(&" ".repeat(
                        width - val.get_min_width() - Self::WALLS_WIDTH - min_key_width - 1,
                    ));
                    row.push_str(&val.to_string());
                    row.push_str(right_edge);
                    rows.push(row);
                }
                rows.push(Self::get_bottom_row(width, Some(min_key_width)));
            }
            VidereCell::Arr(values) => {
                rows.push(Self::get_top_row(width, None, layer));
                for val in values {
                    let right_edge = if let VidereValue::Object | VidereValue::Array = val {
                        from.push(rows.len() + from_offset);
                        Self::CROSS_RIGHT
                    } else {
                        Self::VERTICAL_EDGE
                    };

                    let mut row = String::from(Self::VERTICAL_EDGE);
                    row.push_str(&" ".repeat(width - val.get_min_width() - Self::WALLS_WIDTH));
                    row.push_str(&val.to_string());
                    row.push_str(right_edge);
                    rows.push(row);
                }
                rows.push(Self::get_bottom_row(width, None));
            }
        }

        return (rows, from);
    }
}

#[derive(Debug)]
pub struct VidereLayer {
    cells: Vec<VidereCell>,
}

impl VidereLayer {
    fn new() -> Self {
        Self { cells: Vec::new() }
    }

    fn get_min_width(&self) -> usize {
        self.cells
            .iter()
            .map(|c| c.get_min_width())
            .max()
            .unwrap_or(0)
    }

    fn get_min_height(&self) -> usize {
        self.cells.iter().map(|c| c.get_height()).sum()
    }

    fn get_rows(
        &self,
        height: usize,
        mut connections: LayerConnector,
        layer: usize,
    ) -> (Vec<String>, Vec<usize>) {
        let width = self.get_min_width();

        let mut rows = Vec::new();
        let mut to = Vec::new();
        let mut from = Vec::new();
        for cell in &self.cells {
            to.push(rows.len());
            let (mut cell_rows, mut cell_from) = cell.get_rows(width, rows.len(), layer);
            rows.append(&mut cell_rows);
            from.append(&mut cell_from);
        }

        while rows.len() < height {
            rows.push(" ".repeat(width));
        }

        connections.to = to;
        connections.resolve();
        for (idx, row) in rows.iter_mut().enumerate() {
            let prefix = connections.get_row(idx);
            row.insert_str(0, &prefix);
        }

        return (rows, from);
    }
}

#[derive(Debug)]
pub struct VidereMap {
    layers: Vec<VidereLayer>,
}

impl VidereMap {
    pub fn from_json_obj(obj: Map<String, Value>) -> Self {
        let mut map = VidereMap { layers: Vec::new() };
        map.add_obj_to_layer(0, obj);
        return map;
    }

    fn fill_missing_layer(&mut self, layer: usize) {
        while let None = self.layers.get(layer) {
            self.layers.push(VidereLayer::new());
        }
    }

    pub fn add_obj_to_layer(&mut self, layer: usize, obj: Map<String, Value>) -> usize {
        let mut entries = Vec::new();

        for (key, val) in obj {
            let entry = (key, VidereValue::from_json_val(self, layer, val));
            entries.push(entry);
        }

        self.fill_missing_layer(layer);

        let layer = &mut self.layers[layer];
        layer.cells.push(VidereCell::Obj(entries));
        return layer.cells.len() - 1;
    }

    pub fn add_arr_to_layer(&mut self, layer: usize, arr: Vec<Value>) -> usize {
        let mut entries = Vec::new();

        for val in arr {
            let entry = VidereValue::from_json_val(self, layer, val);
            entries.push(entry);
        }

        self.fill_missing_layer(layer);

        let layer = &mut self.layers[layer];
        layer.cells.push(VidereCell::Arr(entries));
        return layer.cells.len() - 1;
    }

    fn get_height(&self) -> usize {
        self.layers
            .iter()
            .map(|l| l.get_min_height())
            .max()
            .unwrap_or(0)
    }

    pub fn as_table_string(&self) -> String {
        let height = self.get_height();
        let mut layer_strings = Vec::new();

        let mut from = Vec::new();
        for (layer_idx, layer) in self.layers.iter().enumerate() {
            let connections = LayerConnector::new(height, from);
            let (rows, layer_from) = layer.get_rows(height, connections, layer_idx);
            layer_strings.push(rows);
            from = layer_from
        }

        let mut table = String::new();
        for row in 0..height {
            for col in &layer_strings {
                table.push_str(&col[row]);
            }

            table.push('\n');
        }

        return table;
    }
}
