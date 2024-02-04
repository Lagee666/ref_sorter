#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use std::{
    collections::HashMap,
    fs::{self, File},
    io::Read,
};

use eframe::{
    egui::{self},
    emath::Align,
    epaint::Vec2,
};
fn main() -> Result<(), eframe::Error> {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    let mut viewport = egui::ViewportBuilder::default()
        .with_inner_size([840.0, 540.0]) // wide enough for the drag-drop overlay text
        .with_drag_and_drop(true)
        .with_resizable(false);
    viewport.maximize_button = Some(false);
    let options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };

    eframe::run_native(
        "Reference sorter",
        options,
        Box::new(|_cc| Box::<MyApp>::new(MyApp::new(_cc))),
    )
}

// #[derive(Default)]
struct MyApp {
    dropped_files: Vec<egui::DroppedFile>,
    picked_path: Option<String>,
    chinese_table: HashMap<char, i32>,
    reference: String,
    reference_sorted: String,
    numbering: bool,
}
impl MyApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        load_global_font(&cc.egui_ctx);
        MyApp::default()
    }
    fn ref_sort(
        &self,
        file: &mut File,
        numbering: bool,
        chinese_table: HashMap<char, i32>,
    ) -> Vec<String> {
        let mut buffer = String::new();
        file.read_to_string(&mut buffer).unwrap();
        let mut contents: Vec<String> = buffer
            .lines()
            .map(|s| {
                s.chars()
                    .skip_while(|c| !c.is_alphabetic())
                    .collect::<String>()
            })
            // .map(|s| s.to_string())
            .collect::<Vec<_>>();
        // content.sort_by_key(|s|s.chars().find(|x| x.is_alphabetic()));
        let mut english_ref = vec![];
        let mut chinese_ref = vec![];
        for content in &contents {
            if let Some(char) = &content.chars().next() {
                if ('\u{0030}'..='\u{007A}').contains(&char) {
                    english_ref.push(content.clone());
                } else {
                    chinese_ref.push(content.clone());
                }
            }
            
        }
        let mut ref_sort = vec![];
        ref_sort.extend(self.ref_sort_en(english_ref));
        ref_sort.extend(self.ref_sort_zh(chinese_ref));

        if numbering {
            for i in 1..=ref_sort.len() {
                ref_sort[i - 1] = format!("{}. {}", i, ref_sort[i - 1]);
            }
        }
        ref_sort
    }
    fn ref_sort_en(&self, content: Vec<String>) -> Vec<String> {
        let mut content = content;
        content.sort();
        content
    }

    fn ref_sort_zh(&self, content: Vec<String>) -> Vec<String> {
        let mut content = content;
        content.sort_by_key(|s| self.get_stroke(s.chars().next().unwrap()));
        content
    }

    fn get_stroke(&self, chinese: char) -> i32 {
        let chinese_table = &self.chinese_table;
        let chinese = chinese;
        *chinese_table.get(&chinese).unwrap()
    }

    fn get_chinese_table(file: &mut File) -> HashMap<char, i32> {
        let mut buffer = String::new();
        file.read_to_string(&mut buffer).unwrap();
        let contents: Vec<&str> = buffer.lines().skip(6).collect::<Vec<_>>();
        let mut chinese_table: HashMap<char, i32> = HashMap::new();
        for content in contents {
            let data: Vec<&str> = content.split_whitespace().collect::<Vec<_>>();
            let entry = chinese_table.entry(data[0].chars().next().unwrap());
            entry.or_insert(data[6].parse::<i32>().unwrap_or(0));
        }
        chinese_table
    }
}
impl Default for MyApp {
    fn default() -> Self {
        Self {
            dropped_files: vec![],
            picked_path: Some(String::new()),
            chinese_table: Self::get_chinese_table(
                &mut File::open("assets/chinese_unicode_table.txt").unwrap(),
            ),
            reference: String::new(),
            reference_sorted: String::new(),
            numbering: true,
        }
    }
}
impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.label("Drag-and-drop files to the window or click Open file.     Push sort button to sort the reference");

            let mut message = format!("");
            ui.with_layout(egui::Layout::top_down(Align::Center), |ui| {
                ui.horizontal(|ui| {
                    let size = Vec2{x: 110.0, y: 30.0};
                    if ui.add(egui::Button::new("Open file…").min_size(size)).clicked() {
                        if let Some(path) = rfd::FileDialog::new().pick_file() {
                            self.picked_path = Some(path.display().to_string());
                        }
                    }
                    if ui.add(egui::Button::new("Save as").min_size(size)).clicked() {
                        let extensions = ["txt"];
                        let filedialog = rfd::FileDialog::new().add_filter("Text File", &extensions).set_file_name("Sorted_reference");
                        if let Some(path) = filedialog.save_file() {
                            fs::write(path, &self.reference_sorted).unwrap();
                        }
                    }
                    if ui.add(egui::Button::new("Sort").min_size(size)).clicked() {
                        let mut content = vec![];
                        let path = self.picked_path.clone().unwrap_or(String::new());
                        match File::open(path) {
                            Ok(mut file) => {
                                content = self.ref_sort(&mut file, self.numbering, self.chinese_table.clone());
                            }
                            Err(_) => content.push("File open fail.".to_string()),
                        }
                        message = content.join("\n");
                        self.reference_sorted = message;
                    }
                    ui.add(egui::Checkbox::new(&mut self.numbering, "numbering"));
                })
            });


            if let Some(picked_path) = &self.picked_path {
                ui.horizontal(|ui| {
                    ui.label("Picked file:");
                    ui.monospace(picked_path);
                });
                if let Ok(contents) = std::fs::read_to_string(picked_path) {
                    self.reference = contents;
                }
            }
            let reference_scroll_area_id = egui::Id::new("reference_scroll_area");
            let sort_result_scroll_area_id = egui::Id::new("sort_result_scroll_area");
            ui.horizontal_centered(|ui| {
                let size = Vec2 { x: 150.0, y: 450.0 };
                ui.label("Reference: \n");
                egui::ScrollArea::vertical()
                    .max_height(450.0)
                    .id_source(reference_scroll_area_id)
                    .show(ui, |ui| {
                        ui.add(egui::TextEdit::multiline(&mut self.reference).min_size(size));
                    });

                ui.label("Sort result: ");
                egui::ScrollArea::vertical()
                    .max_height(450.0)
                    .id_source(sort_result_scroll_area_id)
                    .show(ui, |ui| {
                        ui.add(
                            egui::TextEdit::multiline(&mut self.reference_sorted).min_size(size),
                        );
                    });
            });

            // Show dropped files (if any):
            if !self.dropped_files.is_empty() {
                ui.group(|ui| {
                    ui.label("Dropped files:");

                    for file in &self.dropped_files {
                        let mut info = if let Some(path) = &file.path {
                            path.display().to_string()
                        } else if !file.name.is_empty() {
                            file.name.clone()
                        } else {
                            "???".to_owned()
                        };

                        let mut additional_info = vec![];
                        if !file.mime.is_empty() {
                            additional_info.push(format!("type: {}", file.mime));
                        }
                        if let Some(bytes) = &file.bytes {
                            additional_info.push(format!("{} bytes", bytes.len()));
                        }
                        if !additional_info.is_empty() {
                            info += &format!(" ({})", additional_info.join(", "));
                        }

                        ui.label(info);
                    }
                });
            }
        });

        preview_files_being_dropped(ctx);

        // Collect dropped files:
        ctx.input(|i| {
            if !i.raw.dropped_files.is_empty() {
                self.dropped_files = i.raw.dropped_files.clone();
            }
        });
    }
}
pub fn load_global_font(ctx: &egui::Context) {
    let mut fonts = eframe::egui::FontDefinitions::default();

    // Install my own font (maybe supporting non-latin characters):
    fonts.font_data.insert(
        "msyh".to_owned(),
        eframe::egui::FontData::from_static(include_bytes!("C:\\Windows\\Fonts\\msyh.ttc")),
    ); // .ttf and .otf supported

    // Put my font first (highest priority):
    fonts
        .families
        .get_mut(&eframe::egui::FontFamily::Proportional)
        .unwrap()
        .insert(0, "msyh".to_owned());

    // Put my font as last fallback for monospace:
    fonts
        .families
        .get_mut(&eframe::egui::FontFamily::Monospace)
        .unwrap()
        .push("msyh".to_owned());

    // let mut ctx = egui::CtxRef::default();
    ctx.set_fonts(fonts);
}

/// Preview hovering files:
fn preview_files_being_dropped(ctx: &egui::Context) {
    use egui::*;
    use std::fmt::Write as _;

    if !ctx.input(|i| i.raw.hovered_files.is_empty()) {
        let text = ctx.input(|i| {
            let mut text = "Dropping files:\n".to_owned();
            for file in &i.raw.hovered_files {
                if let Some(path) = &file.path {
                    write!(text, "\n{}", path.display()).ok();
                } else if !file.mime.is_empty() {
                    write!(text, "\n{}", file.mime).ok();
                } else {
                    text += "\n???";
                }
            }
            text
        });

        let painter =
            ctx.layer_painter(LayerId::new(Order::Foreground, Id::new("file_drop_target")));

        let screen_rect = ctx.screen_rect();
        painter.rect_filled(screen_rect, 0.0, Color32::from_black_alpha(192));
        painter.text(
            screen_rect.center(),
            Align2::CENTER_CENTER,
            text,
            TextStyle::Heading.resolve(&ctx.style()),
            Color32::WHITE,
        );
    }
}

#[cfg(test)]
mod tests {
    use std::fs::File;

    use crate::MyApp;

    #[test]
    fn sort_test() {
        let path = "ref.txt";
        let mut file = File::open(path).unwrap();
        let myapp = MyApp::default();
        let chinese_table = MyApp::get_chinese_table(
            &mut File::open("./assets/chinese_unicode_table.txt").unwrap(),
        );
        let content = myapp.ref_sort(&mut file, false, chinese_table);
        println!("{:?}", content);
    }
    #[test]
    fn sort_zh_test() {
        let path = "ref_zh.txt";
        let mut file = File::open(path).unwrap();
        let myapp = MyApp::default();
        let chinese_table = MyApp::get_chinese_table(
            &mut File::open("./assets/chinese_unicode_table.txt").unwrap(),
        );
        let content = myapp.ref_sort(&mut file, false, chinese_table);
        println!("{:?}", content);
    }
}
