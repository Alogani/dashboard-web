use serde::{Deserialize, Deserializer};
use std::collections::HashMap;

#[derive(Debug)]
pub struct AdminCommands {
    hosts: HashMap<String, String>,
    panels: HashMap<String, String>,
    commands: HashMap<String, Command>,
}

#[derive(Debug, Deserialize)]
pub struct Command {
    pub name: String,
    pub host: String,
    pub panel: String,
    pub command: String,
}

impl<'de> Deserialize<'de> for AdminCommands {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct TempAdminCommands {
            hosts: HashMap<String, String>,
            panels: HashMap<String, String>,
            commands: HashMap<String, Command>,
        }

        let temp = TempAdminCommands::deserialize(deserializer)?;

        // Validate commands
        for (key, command) in &temp.commands {
            if !temp.hosts.contains_key(&command.host) {
                return Err(serde::de::Error::custom(format!(
                    "Host '{}' in command '{}' is not present in hosts",
                    command.host, key
                )));
            }
            if !temp.panels.contains_key(&command.panel) {
                return Err(serde::de::Error::custom(format!(
                    "Panel '{}' in command '{}' is not present in panels",
                    command.panel, key
                )));
            }
        }

        Ok(AdminCommands {
            hosts: temp.hosts,
            panels: temp.panels,
            commands: temp.commands,
        })
    }
}

impl AdminCommands {
    pub fn get_panels(&self) -> &HashMap<String, String> {
        &self.panels
    }

    pub fn get_hosts(&self) -> &HashMap<String, String> {
        &self.hosts
    }

    pub fn get_commands(&self) -> &HashMap<String, Command> {
        &self.commands
    }

    pub fn get_panels_with_commands(&self) -> Vec<(&str, Vec<(&str, &str)>)> {
        self.get_panels()
            .iter()
            .map(|(panel_key, _)| {
                let panel_name = self.panels.get(panel_key).unwrap();
                let commands: Vec<(&str, &str)> = self
                    .get_commands()
                    .iter()
                    .filter(|(_, cmd)| cmd.panel == *panel_key)
                    .map(|(cmd_key, cmd)| (cmd_key.as_str(), cmd.name.as_str()))
                    .collect();
                (panel_name.as_str(), (commands))
            })
            .collect()
    }
}
