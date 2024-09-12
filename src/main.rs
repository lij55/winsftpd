#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
#![allow(rustdoc::missing_crate_level_docs)] // it's an example

use std::io::{BufRead, BufReader, Write};
use std::os::windows::process::CommandExt;
use eframe::{egui, glow};
use std::process::{Command, Stdio, Child};
use regex::Regex;
use winapi::um::winbase::CREATE_NO_WINDOW;


fn main() -> eframe::Result {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([640.0, 240.0]) // wide enough for the drag-drop overlay text
            .with_position([400.0, 400.0])
            .with_fullscreen(false)
            .with_maximized(false)
            .with_drag_and_drop(false),
        ..Default::default()
    };
    eframe::run_native(
        "sftp server",
        options,
        Box::new(|_cc| Ok(Box::<MyApp>::default())),
    )
}

#[derive(Default)]
struct MyApp {
    picked_path: Option<String>,
    console_output: String,
    child_process: Option<Child>,
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {

            match self.picked_path {
                None => {  ui.label("Choose folder to share");
                    if ui.button("Open Dir…").clicked() {
                        if let Some(path) = rfd::FileDialog::new().pick_folder() {
                            self.picked_path = Some(path.display().to_string());
                        }
                    } }
                Some(ref picked_path) => {
                    ui.horizontal(|ui| {
                        ui.label("Picked file:");
                        ui.monospace(picked_path);
                    });


                    match self.child_process {
                        None => {
                            ui.label("fork child process");

                            let exe_bytes = include_bytes!("../sftpgo.exe");
                            let temp_dir = std::env::temp_dir();
                            let exe_path = temp_dir.join("temp_sftpgo.exe");

                            {
                                let mut file = std::fs::File::create(&exe_path).expect("Failed to create temp file");
                                file.write_all(exe_bytes).expect("Failed to write to temp file");
                            }

                            self.child_process = Command::new(&exe_path)
                                .arg("portable")
                                .arg("-d")
                                .arg(picked_path)
                                .arg("-s")
                                .arg("2022")
                                .arg("-p")
                                .arg("changeme")
                                .creation_flags(CREATE_NO_WINDOW) // 隐藏子进程的控制台窗口
                                .stdout(Stdio::piped())
                                .spawn().ok();
                            //println!("{:?}", self.child_process);
                        }
                        _ => {
                            ui.label("Console Output:");

                            // 捕获子进程的标准输出
                            if let Some(stdout) = self.child_process.as_mut().unwrap().stdout.take(){
                                let mut reader = BufReader::new(stdout);

                                // 正则表达式用于去除 ANSI 控制字符
                                let re = Regex::new(r"\x1b\[[0-9;]*m").unwrap();
                                let line = &mut "".to_string();

                                let _ = reader.read_line(line);
                                let cleaned_line = re.replace_all(&line, "").to_string();

                                // 将输出追加到控制台输出中
                                self.console_output.push_str(&cleaned_line);
                                self.console_output.push('\n');
                                //println!("{}", self.console_output);


                            }
                            egui::ScrollArea::vertical().show(ui, |ui| {
                                ui.label(self.console_output.clone());
                            });
                        }
                    }


                },
            }

        });


    }

    fn on_exit(&mut self, _gl: Option<&glow::Context>) {
        // 关闭应用程序时终止子进程
        if let Some(ref mut child) = self.child_process {
            let _ = child.kill();
        }
    }
}
