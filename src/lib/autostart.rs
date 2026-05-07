#![cfg(not(any(target_os = "android", target_os = "ios")))]

use auto_launch::{AutoLaunch, AutoLaunchBuilder};
use std::env::current_exe;

pub type Result<T> = std::result::Result<T, String>;

#[derive(Debug, Default, Copy, Clone)]
pub enum MacosLauncher {
    #[default]
    LaunchAgent,
    AppleScript,
}

pub struct AutoLaunchManager {
    inner: AutoLaunch,
}

impl AutoLaunchManager {
    pub fn new(inner: AutoLaunch) -> Self {
        Self { inner }
    }

    pub fn enable(&self) -> Result<()> {
        self.inner.enable().map_err(|e| e.to_string())
    }

    pub fn disable(&self) -> Result<()> {
        self.inner.disable().map_err(|e| e.to_string())
    }

    pub fn is_enabled(&self) -> Result<bool> {
        self.inner.is_enabled().map_err(|e| e.to_string())
    }
}

#[derive(Default)]
pub struct Builder {
    #[cfg(target_os = "macos")]
    macos_launcher: MacosLauncher,
    args: Vec<String>,
    app_name: Option<String>,
}

impl Builder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn arg<S: Into<String>>(mut self, arg: S) -> Self {
        self.args.push(arg.into());
        self
    }

    pub fn args<I, S>(mut self, args: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        for arg in args {
            self.args.push(arg.into());
        }
        self
    }

    pub fn app_name<S: Into<String>>(mut self, app_name: S) -> Self {
        self.app_name = Some(app_name.into());
        self
    }

    #[cfg(target_os = "macos")]
    pub fn macos_launcher(mut self, launcher: MacosLauncher) -> Self {
        self.macos_launcher = launcher;
        self
    }

    pub fn build(self) -> Result<AutoLaunchManager> {
        let mut builder = AutoLaunchBuilder::new();

        let app_name = self
            .app_name
            .unwrap_or_else(|| "my-app".to_string());

        builder.set_app_name(&app_name);
        builder.set_args(&self.args);

        let current_exe = current_exe().map_err(|e| e.to_string())?;

        #[cfg(windows)]
        {
            builder.set_app_path(&current_exe.display().to_string());
        }

        #[cfg(target_os = "linux")]
        {
            builder.set_app_path(&current_exe.display().to_string());
        }

        #[cfg(target_os = "macos")]
        {
            builder.set_use_launch_agent(matches!(
                self.macos_launcher,
                MacosLauncher::LaunchAgent
            ));

            let exe_path = current_exe
                .canonicalize()
                .map_err(|e| e.to_string())?
                .display()
                .to_string();

            let parts: Vec<&str> = exe_path.split(".app/").collect();

            let app_path = if parts.len() == 2
                && matches!(self.macos_launcher, MacosLauncher::AppleScript)
            {
                format!("{}.app", parts[0])
            } else {
                exe_path
            };

            builder.set_app_path(&app_path);
        }

        let autolaunch = builder.build().map_err(|e| e.to_string())?;

        Ok(AutoLaunchManager::new(autolaunch))
    }
}