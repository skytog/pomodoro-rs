use eframe::egui;
use std::f32::consts::PI;
use std::time::{Duration, Instant};

const WORK_DURATION: u64 = 25 * 60; // 25分
const BREAK_DURATION: u64 = 5 * 60; // 5分
const CIRCLE_RADIUS: f32 = 120.0;

struct PomodoroTimer {
    timer: Option<(Instant, Duration)>,
    remaining: Duration,
    is_break: bool,
    completed_pomodoros: u32,
}

impl Default for PomodoroTimer {
    fn default() -> Self {
        Self {
            timer: None,
            remaining: Duration::from_secs(WORK_DURATION),
            is_break: false,
            completed_pomodoros: 0,
        }
    }
}

impl eframe::App for PomodoroTimer {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if let Some((start, duration)) = self.timer {
            let now = Instant::now();
            let elapsed = now.duration_since(start);
            
            if elapsed >= duration {
                if !self.is_break {
                    self.completed_pomodoros += 1;
                }
                self.timer = None;
                self.is_break = !self.is_break;
                self.remaining = if self.is_break {
                    Duration::from_secs(BREAK_DURATION)
                } else {
                    Duration::from_secs(WORK_DURATION)
                };
            } else {
                self.remaining = duration.saturating_sub(elapsed);
            }
            ctx.request_repaint();
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            let available_size = ui.available_size();
            let center = egui::pos2(
                available_size.x / 2.0,
                available_size.y / 2.0,
            );

            // 背景色の設定
            let rect = ui.max_rect();
            let bg_color = if self.is_break {
                egui::Color32::from_rgb(200, 230, 210) // 休憩時は優しい緑
            } else {
                egui::Color32::from_rgb(230, 210, 200) // 作業時は優しい赤
            };
            ui.painter().rect_filled(rect, 0.0, bg_color);

            // プログレスサークルを描画
            let total_duration = if self.is_break { BREAK_DURATION } else { WORK_DURATION } as f32;
            let progress = self.remaining.as_secs_f32() / total_duration;

            // 外側の円
            ui.painter().circle_stroke(
                center,
                CIRCLE_RADIUS + 5.0,
                egui::Stroke::new(2.0, egui::Color32::from_gray(100)),
            );

            // プログレス円
            let color = if self.is_break {
                egui::Color32::from_rgb(46, 204, 113)
            } else {
                egui::Color32::from_rgb(231, 76, 60)
            };

            ui.painter().circle_filled(center, CIRCLE_RADIUS, egui::Color32::from_gray(240));
            
            // プログレスアークを点で描画
            let start_angle = -PI / 2.0; // 12時の位置から開始
            let points_count = 100;
            let mut prev_point = None;
            for i in 0..=points_count {
                let angle = start_angle + (2.0 * PI * (1.0 - progress) * i as f32 / points_count as f32);
                let point = egui::pos2(
                    center.x + CIRCLE_RADIUS * angle.cos(),
                    center.y + CIRCLE_RADIUS * angle.sin(),
                );
                if let Some(prev) = prev_point {
                    ui.painter().line_segment(
                        [prev, point],
                        egui::Stroke::new(4.0, color),
                    );
                }
                prev_point = Some(point);
            }

            // 中央にテキストを配置
            let minutes = self.remaining.as_secs() / 60;
            let seconds = self.remaining.as_secs() % 60;
            let text = format!("{:02}:{:02}", minutes, seconds);
            
            let font_size = 32.0;
            let text_color = egui::Color32::from_gray(60);
            let galley = ui.painter().layout_no_wrap(
                text.to_string(),
                egui::FontId::proportional(font_size),
                text_color,
            );
            let text_pos = center - galley.size() / 2.0;
            ui.painter().galley(text_pos, galley);

            // 状態テキスト
            let status_text = if self.is_break { "Break Time" } else { "Focus Time" };
            let status_galley = ui.painter().layout_no_wrap(
                status_text.to_string(),
                egui::FontId::proportional(24.0),
                text_color,
            );
            let status_pos = egui::pos2(
                center.x - status_galley.size().x / 2.0,
                center.y - CIRCLE_RADIUS - 40.0,
            );
            ui.painter().galley(status_pos, status_galley);

            // コントロールボタンを配置
            let button_area = egui::Rect::from_center_size(
                egui::pos2(center.x, center.y + CIRCLE_RADIUS + 40.0),
                egui::vec2(300.0, 40.0),
            );
            let response = ui.allocate_ui_at_rect(button_area, |ui| {
                ui.horizontal_centered(|ui| {
                    ui.spacing_mut().item_spacing.x = 10.0;
                    ui.style_mut().visuals.widgets.inactive.expansion = 2.0;
                    ui.style_mut().visuals.widgets.hovered.expansion = 3.0;
                    ui.style_mut().visuals.widgets.active.expansion = 1.0;

                    let button_height = 36.0;
                    if self.timer.is_none() {
                        let start_button = egui::Button::new(
                            egui::RichText::new("▶ Start")
                                .size(20.0)
                                .color(egui::Color32::from_rgb(46, 204, 113))
                        ).min_size(egui::vec2(120.0, button_height));

                        if ui.add(start_button).clicked() {
                            self.timer = Some((Instant::now(), self.remaining));
                        }
                    } else {
                        let pause_button = egui::Button::new(
                            egui::RichText::new("⏸ Pause")
                                .size(20.0)
                                .color(egui::Color32::from_rgb(230, 126, 34))
                        ).min_size(egui::vec2(120.0, button_height));

                        if ui.add(pause_button).clicked() {
                            self.timer = None;
                        }
                    }

                    let reset_button = egui::Button::new(
                        egui::RichText::new("↺ Reset")
                            .size(20.0)
                            .color(egui::Color32::from_rgb(231, 76, 60))
                    ).min_size(egui::vec2(120.0, button_height));

                    if ui.add(reset_button).clicked() {
                        self.timer = None;
                        self.remaining = if self.is_break {
                            Duration::from_secs(BREAK_DURATION)
                        } else {
                            Duration::from_secs(WORK_DURATION)
                        };
                    }
                });
            });

            // 完了したポモドーロの表示
            let pomodoro_area = egui::Rect::from_center_size(
                egui::pos2(center.x, response.response.rect.bottom() + 40.0),
                egui::vec2(200.0, 30.0),
            );
            ui.allocate_ui_at_rect(pomodoro_area, |ui| {
                ui.horizontal_centered(|ui| {
                    for i in 0..4 {
                        let color = if i < (self.completed_pomodoros % 4) as usize {
                            egui::Color32::from_rgb(231, 76, 60)
                        } else {
                            egui::Color32::from_gray(200)
                        };
                        ui.painter().circle_filled(
                            ui.next_widget_position() + egui::vec2(12.0, 10.0),
                            10.0, // インジケーターのサイズを大きく
                            color,
                        );
                        ui.add_space(35.0); // インジケーター間の間隔を広げる
                    }
                });
            });
        });
    }
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([400.0, 600.0])
            .with_title("Pomodoro Timer"),
        ..Default::default()
    };
    
    eframe::run_native(
        "Pomodoro Timer",
        options,
        Box::new(|_cc| Box::new(PomodoroTimer::default())),
    )
}
