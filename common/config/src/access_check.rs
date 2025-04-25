use std::collections::HashMap;

use serde::{Deserialize, Deserializer};

use crate::AppConfig;

pub fn access_rules_deserialize<'de, D>(
    deserializer: D,
) -> Result<HashMap<String, Vec<(String, Vec<String>)>>, D::Error>
where
    D: Deserializer<'de>,
{
    let raw_map = HashMap::<String, Vec<String>>::deserialize(deserializer)?;
    let mut result: HashMap<String, Vec<(String, Vec<String>)>> = HashMap::new();

    for (key, allowed_users) in raw_map {
        let (subdomain, route) = parse_key(&key);
        let routes = result.entry(subdomain).or_insert_with(Vec::new);
        routes.push((route, allowed_users));
    }

    // Sort each subdomain's routes by length in descending order
    for routes in result.values_mut() {
        routes.sort_by(|(a, _), (b, _)| b.len().cmp(&a.len()));
    }

    Ok(result)
}

// Helper function to parse a key into subdomain and route
fn parse_key(key: &str) -> (String, String) {
    if let Some(idx) = key.find('@') {
        let subdomain = key[..idx].to_string();
        let route = key[idx + 1..].to_string();
        (subdomain, route)
    } else {
        // If there's no '@', it's a route without a subdomain
        ("".to_string(), key.to_string())
    }
}

impl AppConfig {
    pub fn is_access_allowed(
        &self,
        subdomain: Option<&str>,
        route: &str,
        username: Option<&str>,
    ) -> bool {
        let subdomain = subdomain.unwrap_or("");
        if let Some(allowed_routes) = self.access_rules.get(subdomain) {
            is_route_allowed_impl(route, username, allowed_routes)
        } else {
            tracing::warn!("No access rules found for subdomain {}", subdomain);
            false
        }
    }
}

fn is_route_allowed_impl(
    route: &str,
    username: Option<&str>,
    allowed_routes: &Vec<(String, Vec<String>)>,
) -> bool {
    // First, check for exact matches which take highest precedence
    // Routes are sorted by length in descending order
    for (allowed_route, allowed_users) in allowed_routes {
        if route == allowed_route
            || (allowed_route.ends_with('/') && route.starts_with(allowed_route))
            || (allowed_route.ends_with('*')
                && route.starts_with(allowed_route.trim_end_matches('*')))
        {
            if allowed_users.contains(&"*".to_string())
                || matches!(username, Some(username) if allowed_users.contains(&username.to_string()))
            {
                return true;
            } else {
                return false;
            }
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::access_rules_deserialize;
    use super::is_route_allowed_impl;
    use serde::Deserialize;
    use std::collections::HashMap;
    use toml;

    #[derive(Deserialize)]
    struct TestConfig {
        #[serde(deserialize_with = "access_rules_deserialize")]
        access_rules: HashMap<String, Vec<(String, Vec<String>)>>,
    }

    #[test]
    fn test_access_rules_deserializer() {
        // Intentionally put paths in mixed order to test sorting
        let config_str = r#"
        [access_rules]
        "/" = ["*"]
        "/action_dashboard/cmd/router_poweroff" = ["admin"]
        "/static/css/" = ["*"]
        "vaultwarden@/" = ["*"]
        "vaultwarden@/admin" = ["desktop"]
        "forgejo@/" = ["desktop", "laptop"]
        "incus@/" = ["desktop"]
        "/api/v1/users" = ["admin", "manager"]
        "/api" = ["*"]
        "#;

        let config: TestConfig = toml::from_str(config_str).unwrap();

        // Check the empty subdomain (global routes)
        let empty_subdomain = config.access_rules.get("").unwrap();
        assert_eq!(empty_subdomain.len(), 5);

        // Verify routes are sorted by length in descending order
        for i in 0..empty_subdomain.len() - 1 {
            assert!(
                empty_subdomain[i].0.len() >= empty_subdomain[i + 1].0.len(),
                "Routes should be sorted by length in descending order: '{}' should come before '{}'",
                empty_subdomain[i].0,
                empty_subdomain[i + 1].0
            );
        }

        // Check specific routes in the expected order
        assert_eq!(
            empty_subdomain[0].0,
            "/action_dashboard/cmd/router_poweroff"
        );
        assert_eq!(empty_subdomain[0].1, vec!["admin"]);

        assert_eq!(empty_subdomain[1].0, "/api/v1/users");
        assert_eq!(empty_subdomain[1].1, vec!["admin", "manager"]);

        assert_eq!(empty_subdomain[2].0, "/static/css/");
        assert_eq!(empty_subdomain[2].1, vec!["*"]);

        assert_eq!(empty_subdomain[3].0, "/api");
        assert_eq!(empty_subdomain[3].1, vec!["*"]);

        assert_eq!(empty_subdomain[4].0, "/");
        assert_eq!(empty_subdomain[4].1, vec!["*"]);

        // Check the "incus" subdomain
        let incus = config.access_rules.get("incus").unwrap();
        assert_eq!(incus.len(), 1);
        assert_eq!(incus[0].0, "/");
        assert_eq!(incus[0].1, vec!["desktop"]);

        // Check the "forgejo" subdomain
        let forgejo = config.access_rules.get("forgejo").unwrap();
        assert_eq!(forgejo.len(), 1);
        assert_eq!(forgejo[0].0, "/");
        assert_eq!(forgejo[0].1, vec!["desktop", "laptop"]);

        // Check the "vaultwarden" subdomain
        let vaultwarden = config.access_rules.get("vaultwarden").unwrap();
        assert_eq!(vaultwarden.len(), 2);

        // Verify vaultwarden routes are sorted by length in descending order
        assert!(
            vaultwarden[0].0.len() >= vaultwarden[1].0.len(),
            "Vaultwarden routes should be sorted by length in descending order"
        );

        assert_eq!(vaultwarden[0].0, "/admin");
        assert_eq!(vaultwarden[0].1, vec!["desktop"]);
        assert_eq!(vaultwarden[1].0, "/");
        assert_eq!(vaultwarden[1].1, vec!["*"]);
    }

    #[test]
    fn test_is_route_allowed_direct() {
        // Create a vector of (route, allowed_users) pairs
        // Routes are sorted by length in descending order as the deserializer would do
        let allowed_routes = vec![
            (
                "/action_dashboard/cmd/router_poweroff".to_string(),
                vec!["admin".to_string()],
            ),
            ("/static/css/".to_string(), vec!["*".to_string()]),
            ("/".to_string(), vec!["*".to_string()]),
        ];

        // Test routes with wildcard access
        assert!(is_route_allowed_impl("/", None, &allowed_routes));
        assert!(is_route_allowed_impl(
            "/",
            Some("any_user"),
            &allowed_routes
        ));
        assert!(is_route_allowed_impl(
            "/static/css/style.css",
            None,
            &allowed_routes
        ));

        // Test route with specific user access
        assert!(!is_route_allowed_impl(
            "/action_dashboard/cmd/router_poweroff",
            None,
            &allowed_routes
        ));
        assert!(!is_route_allowed_impl(
            "/action_dashboard/cmd/router_poweroff",
            Some("user"),
            &allowed_routes
        ));
        assert!(is_route_allowed_impl(
            "/action_dashboard/cmd/router_poweroff",
            Some("admin"),
            &allowed_routes
        ));

        // Test path matching with trailing slash
        let allowed_routes_with_slash = vec![
            ("/protected/area/".to_string(), vec!["admin".to_string()]),
            ("/public/".to_string(), vec!["*".to_string()]),
        ];

        assert!(is_route_allowed_impl(
            "/public/file.txt",
            None,
            &allowed_routes_with_slash
        ));
        assert!(!is_route_allowed_impl(
            "/protected/area/secret.txt",
            None,
            &allowed_routes_with_slash
        ));
        assert!(is_route_allowed_impl(
            "/protected/area/secret.txt",
            Some("admin"),
            &allowed_routes_with_slash
        ));

        // Test wildcard path matching
        let allowed_routes_with_wildcard = vec![
            ("/api/admin*".to_string(), vec!["admin".to_string()]),
            ("/api/public*".to_string(), vec!["*".to_string()]),
        ];

        assert!(is_route_allowed_impl(
            "/api/public/endpoint",
            None,
            &allowed_routes_with_wildcard
        ));
        assert!(!is_route_allowed_impl(
            "/api/admin/users",
            None,
            &allowed_routes_with_wildcard
        ));
        assert!(is_route_allowed_impl(
            "/api/admin/users",
            Some("admin"),
            &allowed_routes_with_wildcard
        ));

        // Test multiple allowed users
        let allowed_routes_multiple_users = vec![(
            "/dashboard".to_string(),
            vec!["admin".to_string(), "manager".to_string()],
        )];

        assert!(!is_route_allowed_impl(
            "/dashboard",
            None,
            &allowed_routes_multiple_users
        ));
        assert!(is_route_allowed_impl(
            "/dashboard",
            Some("admin"),
            &allowed_routes_multiple_users
        ));
        assert!(is_route_allowed_impl(
            "/dashboard",
            Some("manager"),
            &allowed_routes_multiple_users
        ));
        assert!(!is_route_allowed_impl(
            "/dashboard",
            Some("user"),
            &allowed_routes_multiple_users
        ));
    }
}
