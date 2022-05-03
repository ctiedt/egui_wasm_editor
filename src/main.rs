use std::{
    io::{Read, Write},
    process::Stdio,
};

use eframe::CreationContext;
use rfd::FileDialog;

static DEFAULT_CODE: &str = r#"int main() {
    return 0;
}"#;

struct WasmEditor {
    code: String,
    code_output: String,
}

impl WasmEditor {
    fn new(_cc: &CreationContext) -> Self {
        Self {
            code: DEFAULT_CODE.to_owned(),
            code_output: String::new(),
        }
    }

    fn compile(&self) -> anyhow::Result<String> {
        let mut src_tempfile = tempfile::NamedTempFile::new()?;

        src_tempfile.write_all(self.code.as_bytes())?;

        let compile_cmd = std::process::Command::new("clang")
            .args(&[
                "--target=wasm32",
                "--no-standard-libraries",
                "-Wl,--no-entry",
                "-Wl,--import-undefined",
                "-Wl,--export-dynamic",
                "-o",
                "/dev/stdout",
                "-x",
                "c",
            ])
            .arg(src_tempfile.path())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()?;

        let compiler_stdout = compile_cmd.stdout.unwrap();

        let wasm2wat_cmd = std::process::Command::new("wasm2wat")
            .arg("-")
            .stdin(compiler_stdout)
            .output()?;

        let output = wasm2wat_cmd.stdout;

        Ok(String::from_utf8(output)?)
    }
}

impl eframe::App for WasmEditor {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("Source Code", |ui| {
                    if ui.button("Save").clicked() {
                        if let Some(path) = FileDialog::new().save_file() {
                            let mut file = std::fs::File::create(path).unwrap();
                            file.write_all(self.code.as_bytes()).unwrap();
                        }
                    }
                    if ui.button("Load").clicked() {
                        if let Some(path) = FileDialog::new().pick_file() {
                            let mut file = std::fs::File::open(path).unwrap();
                            self.code.clear();
                            file.read_to_string(&mut self.code).unwrap();
                        }
                    }
                });
                ui.menu_button("WASM Code", |ui| {
                    if ui.button("Save WASM").clicked() {
                        if let Some(path) = FileDialog::new()
                            .add_filter("WebAssembly", &["wasm"])
                            .save_file()
                        {
                            let mut src_tempfile = tempfile::NamedTempFile::new().unwrap();

                            src_tempfile.write_all(self.code.as_bytes()).unwrap();

                            std::process::Command::new("clang")
                                .args(&[
                                    "--target=wasm32",
                                    "--no-standard-libraries",
                                    "-Wl,--no-entry",
                                    "-Wl,--import-undefined",
                                    "-Wl,--export-dynamic",
                                    "-x",
                                    "c",
                                    "-o",
                                ])
                                .arg(path)
                                .arg(src_tempfile.path())
                                .stdin(Stdio::piped())
                                .spawn()
                                .unwrap()
                                .wait()
                                .unwrap();

                            std::mem::drop(src_tempfile);
                        }
                    }
                    if ui.button("Save WAT").clicked() {
                        if let Some(path) = FileDialog::new().save_file() {
                            let mut file = std::fs::File::create(path).unwrap();
                            file.write_all(self.code_output.as_bytes()).unwrap();
                        }
                    }
                });
                if ui.button("Compile").clicked() {
                    self.code_output = self.compile().unwrap();
                }
                if ui.button("Quit").clicked() {
                    frame.quit();
                }
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            let mut ui = ui.child_ui(ui.max_rect(), egui::Layout::left_to_right());
            let editor_size = egui::Vec2::new(ui.available_width() / 2.0, ui.available_height());
            egui::ScrollArea::vertical().show(&mut ui, |ui| {
                let _editor = ui.add_sized(
                    editor_size,
                    egui::text_edit::TextEdit::multiline(&mut self.code).code_editor(),
                );

                let _output = ui.add_sized(
                    editor_size,
                    egui::text_edit::TextEdit::multiline(&mut self.code_output.as_str())
                        .code_editor(),
                );
            });
        });
    }
}

fn main() {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "WASM Editor",
        options,
        Box::new(|cc| Box::new(WasmEditor::new(cc))),
    );
}
