use std::collections::HashMap;

use serde::{Deserialize, Deserializer};
use utils::indexed_vector::IndexedVector;
use utils::string_tuple_vec;

#[derive(Debug)]
pub struct AdminConsole {
    pub hosts: IndexedVector<HostId, HostInfo>,
    pub panels: IndexedVector<PanelId, PanelInfo>,
    pub actions: IndexedVector<ActionId, ActionInfo>,
    pub actions_lookup: HashMap<String, ActionId>,
}

impl AdminConsole {
    fn new() -> Self {
        Self {
            hosts: IndexedVector::new(),
            panels: IndexedVector::new(),
            actions: IndexedVector::new(),
            actions_lookup: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PanelId(usize);

impl From<PanelId> for usize {
    fn from(id: PanelId) -> Self {
        id.0
    }
}

impl From<usize> for PanelId {
    fn from(id: usize) -> Self {
        PanelId(id)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActionId(usize);

impl From<ActionId> for usize {
    fn from(id: ActionId) -> Self {
        id.0
    }
}

impl From<usize> for ActionId {
    fn from(id: usize) -> Self {
        ActionId(id)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HostId(usize);

impl From<HostId> for usize {
    fn from(id: HostId) -> Self {
        id.0
    }
}

impl From<usize> for HostId {
    fn from(id: usize) -> Self {
        HostId(id)
    }
}

#[derive(Debug)]
pub struct PanelInfo {
    pub pretty_name: String,
    pub actions: Vec<ActionId>,
}

#[derive(Debug)]
pub struct ActionInfo {
    pub url_name: String,
    pub pretty_name: String,
    pub command: String,
    pub host: HostId,
}

#[derive(Debug)]
pub struct HostInfo {
    pub address: String,
}

impl AdminConsole {
    pub fn get_action_by_url(&self, url_name: &str) -> Option<&ActionInfo> {
        self.actions_lookup
            .get(url_name)
            .and_then(|id| self.actions.get(id))
    }
}

impl<'de> Deserialize<'de> for AdminConsole {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct CommandInfoDes {
            name: String,
            host: String,
            panel: String,
            command: String,
        }

        #[derive(Deserialize)]
        struct TempAdminCommands {
            #[serde(with = "string_tuple_vec")]
            hosts: Vec<(String, String)>,
            #[serde(with = "string_tuple_vec")]
            panels: Vec<(String, String)>,
            #[serde(with = "string_tuple_vec")]
            commands: Vec<(String, CommandInfoDes)>,
        }

        let temp = TempAdminCommands::deserialize(deserializer)?;
        let mut builder = AdminConsole::new();
        for (idx, (_, address)) in temp.hosts.iter().enumerate() {
            builder.hosts.insert(
                HostId(idx),
                HostInfo {
                    address: address.to_string(),
                },
            );
        }

        for (idx, (_, pretty_name)) in temp.panels.iter().enumerate() {
            builder.panels.insert(
                PanelId(idx),
                PanelInfo {
                    pretty_name: pretty_name.to_string(),
                    actions: Vec::new(),
                },
            );
        }

        for (
            idx,
            (
                url_name,
                CommandInfoDes {
                    name: pretty_name,
                    host,
                    panel,
                    command,
                },
            ),
        ) in temp.commands.into_iter().enumerate()
        {
            let host_id = temp
                .hosts
                .iter()
                .enumerate()
                .find(|(_, (host_name, _))| *host_name == host)
                .map(|(idx, _)| HostId(idx))
                .ok_or_else(|| {
                    serde::de::Error::custom(format!(
                        "Host '{:?}' in command '{:?}' is not present in hosts",
                        host, pretty_name
                    ))
                })?;

            let panel_id = temp
                .panels
                .iter()
                .enumerate()
                .find(|(_, (panel_name, _))| *panel_name == panel)
                .map(|(idx, _)| PanelId(idx))
                .ok_or_else(|| {
                    serde::de::Error::custom(format!(
                        "Panel '{:?}' in command '{:?}' is not present in panels",
                        panel, pretty_name
                    ))
                })?;

            let command_id = ActionId(idx);
            builder
                .panels
                .get_mut(&panel_id)
                .unwrap()
                .actions
                .push(command_id.clone());

            let command_info = ActionInfo {
                url_name: url_name.clone(),
                pretty_name: pretty_name.clone(),
                command: command.clone(),
                host: host_id,
            };

            builder.actions_lookup.insert(url_name, command_id.clone());
            builder.actions.insert(command_id, command_info);
        }

        Ok(builder)
    }
}
