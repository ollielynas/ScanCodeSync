use std::{process::Command, sync::Arc, time::{Duration, Instant}};

use egui_macroquad::egui::{self, Ui, ahash::{HashMap, HashSet}, mutex::Mutex};
use exiftool::ExifTool;
use ffmpeg_sidecar::command::FfmpegCommand;


#[derive(Clone)]
pub struct SharedCommandPool {
    inner: Arc<Mutex<CommandPool>>,
    id: u64,
}



impl SharedCommandPool {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(CommandPool::new())),
            id: 0,
        }
    }
    /// dont forget to call .with id before .clone because otherwise all of the commands will get pruned.
    pub fn with_id(&self, id: u64) -> SharedCommandPool {
        SharedCommandPool {
            inner: self.inner.clone(),
            id
        }
    }

    pub fn get_exif_command(&self) -> anyhow::Result<ExifTool> {
        // Check for an existing tool under the lock
        let existing = {
            let mut pool = self.inner.lock();
            pool.get_exif_command(self.id)
        }; // <-- lock dropped here before the slow ExifTool::new()

        if let Some(tool) = existing {
            return Ok(tool);
        }

        Ok(ExifTool::new()?) // constructed outside the lock
    }
    pub fn get_ffmpeg_command(&self) -> anyhow::Result<FfmpegCommand> {
        // Check for an existing tool under the lock
        let existing = {
            let mut pool = self.inner.lock();
            pool.get_ffmpeg_command(self.id)
        };

        if let Some(tool) = existing {
            return Ok(tool);
        }

        Ok(FfmpegCommand::new()) // constructed outside the lock
    }
    pub fn get_magick_command(&self) -> anyhow::Result<Command> {
        // Check for an existing tool under the lock
        let existing = {
            let mut pool = self.inner.lock();
            pool.get_magick_command(self.id)
        };

        if let Some(tool) = existing {
            return Ok(tool);
        }

        let mut cmd = Command::new("magick");
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            cmd.creation_flags(0x08000000);
        }

        Ok(cmd) // constructed outside the lock
    }


    pub fn return_exif_command(&self, tool: ExifTool) {
        self.inner.lock().return_exif_command(self.id, tool);
    }
    pub fn return_ffmpeg_command(&self, tool: FfmpegCommand) {
        self.inner.lock().return_ffmpeg_command(self.id, tool);
    }
    pub fn return_magick_command(&self, tool: Command) {
        self.inner.lock().return_magick_command(self.id, tool);
    }

    pub fn prune_commands(&self, task_ids: &HashSet<u64>) {
        self.inner.lock().prune_commands(task_ids);
    }
}


struct CommandPool {
    exiftool: Vec<ExifToolHolder>,
    magick: Vec<MagickHolder>,
    ffmpeg: Vec<FfmpegHolder>,
}


struct ExifToolHolder {
    cmd: Option<ExifTool>,
    task_id: u64,
}
struct FfmpegHolder {
    cmd: Option<FfmpegCommand>,
    task_id: u64,
}
struct MagickHolder {
    cmd: Option<Command>,
    task_id: u64,
}

impl CommandPool {

    pub fn new() -> CommandPool {
        return CommandPool {
            exiftool: vec![],
            magick: vec![],
            ffmpeg: vec![],
        }
    }

    pub fn prune_commands(&mut self, task_ids: &HashSet<u64>) {
        self.exiftool.retain(|x| task_ids.contains(&x.task_id));
    }

    pub fn get_exif_command(&mut self, task_id: u64) -> Option<ExifTool> {
        for tool in &mut self.exiftool {
            if tool.task_id == task_id {
                if let Some(t) = tool.cmd.take() {
                    return Some(t);
                }
            }
        }
        self.exiftool.push(ExifToolHolder { cmd: None, task_id });


        return None;
    }
    pub fn get_magick_command(&mut self, _task_id: u64) -> Option<Command> {

        // this is not yet supported

        // for tool in &mut self.magick {
        //     if tool.task_id == task_id {
        //         if let Some(t) = tool.cmd.take() {
        //             return Some(t);
        //         }
        //     }
        // }
        // self.magick.push(MagickHolder { cmd: None, task_id });


        return None;
    }
    pub fn get_ffmpeg_command(&mut self, _task_id: u64) -> Option<FfmpegCommand> {

        // ffmpeg is not currently supported

        // for tool in &mut self.exiftool {
        //     if tool.task_id == task_id {
        //         if let Some(t) = tool.cmd.take() {
        //             return Some(t);
        //         }
        //     }
        // }
        // self.exiftool.push(ExifToolHolder { cmd: None, task_id });


        return None;
    }
    pub fn return_exif_command(&mut self, task_id: u64, tool: ExifTool) {
        for t in &mut self.exiftool {
            if t.task_id == task_id {
                if t.cmd.is_none() {
                    t.cmd = Some(tool);
                    return
                }
            }
        }
        self.exiftool.push(
            ExifToolHolder { cmd: Some(tool), task_id }
        );
    }
    pub fn return_magick_command(&mut self, task_id: u64, tool: Command) {

        drop(tool);
        return;

        // for t in &mut self.magick {
        //     if t.task_id == task_id {
        //         if t.cmd.is_none() {
        //             t.cmd = Some(tool);
        //             return
        //         }
        //     }
        // }
        // self.magick.push(
        //     MagickHolder { cmd: Some(tool), task_id }
        // );
    }
    pub fn return_ffmpeg_command(&mut self, task_id: u64, tool: FfmpegCommand) {
        // for t in &mut self.ffmpeg {
        //     if t.task_id == task_id {
        //         if t.cmd.is_none() {
        //             t.cmd = Some(tool);
        //             return
        //         }
        //     }
        // }
        // self.ffmpeg.push(
        //     Ffmpeg { cmd: Some(tool), task_id }
        // );
    }
}



pub struct SharedCommandPoolUiState {
    last_updated: Instant,
    pool: SharedCommandPool,
    /// HashMap<TaskId, (used, unused)>
    exif_tools: HashMap<u64, (i32, i32)>,
    ffmpeg_tools: HashMap<u64, (i32, i32)>,
    magick_tools: HashMap<u64, (i32, i32)>,
}

impl SharedCommandPoolUiState {
    pub fn render(&mut self, ui: &mut Ui) -> anyhow::Result<()> {
        // I might have to adjust this
        if Instant::now().duration_since(self.last_updated) > Duration::from_secs_f32(0.4) {
            self.last_updated = Instant::now();
            let pool = self.pool.inner.lock();
            self.exif_tools.clear();
            for t in &pool.exiftool {
                if !self.exif_tools.contains_key(&t.task_id){
                    self.exif_tools.insert(t.task_id, (0,0));
                };
                let (used, unused) = self.exif_tools.get_mut(&t.task_id).ok_or(anyhow::anyhow!("failed to get value"))?;
                if t.cmd.is_some() {
                    *used += 1;
                }else {
                    *unused += 1;
                }
            }
            self.magick_tools.clear();
            for t in &pool.magick {
                if !self.magick_tools.contains_key(&t.task_id){
                    self.magick_tools.insert(t.task_id, (0,0));
                };
                let (used, unused) = self.magick_tools.get_mut(&t.task_id).ok_or(anyhow::anyhow!("failed to get value"))?;
                if t.cmd.is_some() {
                    *used += 1;
                }else {
                    *unused += 1;
                }
            }
            self.ffmpeg_tools.clear();
            for t in &pool.ffmpeg {
                if !self.ffmpeg_tools.contains_key(&t.task_id){
                    self.ffmpeg_tools.insert(t.task_id, (0,0));
                };
                let (used, unused) = self.ffmpeg_tools.get_mut(&t.task_id).ok_or(anyhow::anyhow!("failed to get value"))?;
                if t.cmd.is_some() {
                    *used += 1;
                }else {
                    *unused += 1;
                }
            }
        }

        egui::Grid::new("commands").show(ui, |ui| {
            ui.label("Command");
            ui.label("Task Id");
            ui.label("busy");
            ui.label("idle");
            ui.end_row();

            for (key, value) in self.exif_tools.iter() {
                ui.label("exiftool");
                ui.label((key%999).to_string());
                ui.label(value.0.to_string());
                ui.label(value.1.to_string());
                ui.end_row();
            }
            for (key, value) in self.ffmpeg_tools.iter() {
                ui.label("ffmpeg");
                ui.label((key%999).to_string());
                ui.label(value.0.to_string());
                ui.label(value.1.to_string());
                ui.end_row();
            }
            for (key, value) in self.magick_tools.iter() {
                ui.label("magick");
                ui.label((key%999).to_string());
                ui.label(value.0.to_string());
                ui.label(value.1.to_string());
                ui.end_row();
            }

        });


        Ok(())
    }

    pub fn new(pool: SharedCommandPool) -> SharedCommandPoolUiState {
        SharedCommandPoolUiState {
            last_updated: Instant::now(), pool,
            exif_tools: HashMap::default(),
            ffmpeg_tools: HashMap::default(),
            magick_tools: HashMap::default(),
        }
    }
}
