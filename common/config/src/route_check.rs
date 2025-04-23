use std::collections::HashMap;

use crate::AppConfig;

impl AppConfig {
    pub fn is_route_allowed(&self, route: &str, username: Option<&str>) -> bool {
        return is_route_allowed_impl(route, username, &self.allowed_routes);
    }

    pub fn is_subdomain_allowed(&self, subdomain: &str, username: Option<&str>) -> bool {
        if let Some(allowed_users) = self.allowed_subdomains.get(subdomain) {
            // If username is None, only allow access if "*" is in allowed_users
            if let Some(username) = username {
                return allowed_users.contains(&username.to_string())
                    || allowed_users.contains(&"*".to_string());
            } else {
                return allowed_users.contains(&"*".to_string());
            }
        }
        false
    }
}

fn is_route_allowed_impl(
    route: &str,
    username: Option<&str>,
    allowed_routes: &HashMap<String, Vec<String>>,
) -> bool {
    // First, check for exact matches which take highest precedence
    if let Some(allowed_users) = allowed_routes.get(route) {
        // If username is None, only allow access if "*" is in allowed_users
        if let Some(username) = username {
            if allowed_users.contains(&username.to_string())
                || allowed_users.contains(&"*".to_string())
            {
                return true;
            }
        } else if allowed_users.contains(&"*".to_string()) {
            return true;
        }
        // If we found an exact match but user doesn't have permission, deny access
        // This prevents parent paths (including "/") from granting access
        return false;
    }

    // Then check for parent paths with proper path segment boundaries
    // Sort paths by length in descending order to check most specific paths first
    let mut paths: Vec<(&String, &Vec<String>)> = allowed_routes.iter().collect();
    paths.sort_by(|(a, _), (b, _)| b.len().cmp(&a.len()));

    for (path, allowed_users) in paths {
        // Check if this is a parent path with proper path boundary
        if path.ends_with('/') && route.starts_with(path) {
            // Path ends with slash, so it's a directory-like path
            if let Some(username) = username {
                if allowed_users.contains(&username.to_string())
                    || allowed_users.contains(&"*".to_string())
                {
                    return true;
                }
            } else if allowed_users.contains(&"*".to_string()) {
                return true;
            }
            // If we found a matching parent path but user doesn't have permission, deny access
            return false;
        } else if !path.is_empty() && route.starts_with(&format!("{}/", path)) {
            // Path doesn't end with slash, ensure we're checking a proper subdirectory
            // with a path separator and that path is not empty
            if let Some(username) = username {
                if allowed_users.contains(&username.to_string())
                    || allowed_users.contains(&"*".to_string())
                {
                    return true;
                }
            } else if allowed_users.contains(&"*".to_string()) {
                return true;
            }
            // If we found a matching parent path but user doesn't have permission, deny access
            return false;
        }
    }

    // Special case for root path "/" - only check if no other path matched
    if let Some(allowed_users) = allowed_routes.get("/") {
        if let Some(username) = username {
            return allowed_users.contains(&username.to_string())
                || allowed_users.contains(&"*".to_string());
        } else {
            return allowed_users.contains(&"*".to_string());
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_routes() -> HashMap<String, Vec<String>> {
        let mut routes = HashMap::new();
        routes.insert("/".to_string(), vec!["*".to_string()]);
        routes.insert("/admin".to_string(), vec!["admin".to_string()]);
        routes.insert(
            "/users/".to_string(),
            vec!["user1".to_string(), "user2".to_string()],
        );
        routes.insert("/users/user2/".to_string(), vec!["user2".to_string()]);
        routes.insert("/public".to_string(), vec!["*".to_string()]);
        routes.insert("/api/v1/".to_string(), vec!["api_user".to_string()]);
        routes.insert("/api/v1/admin/".to_string(), vec!["admin".to_string()]);
        routes.insert(
            "/api/v2/".to_string(),
            vec!["api_user".to_string(), "admin".to_string()],
        );
        routes
    }

    #[test]
    fn test_root_access() {
        let routes = create_test_routes();
        assert!(is_route_allowed_impl("/", Some("any_user"), &routes));
        assert!(is_route_allowed_impl("/", None, &routes));
    }

    #[test]
    fn test_exact_match() {
        let routes = create_test_routes();
        assert!(is_route_allowed_impl("/admin", Some("admin"), &routes));
        assert!(!is_route_allowed_impl("/admin", Some("user1"), &routes));
        assert!(!is_route_allowed_impl("/admin", None, &routes));
    }

    #[test]
    fn test_parent_path_with_slash() {
        let routes = create_test_routes();
        assert!(is_route_allowed_impl(
            "/users/profile",
            Some("user1"),
            &routes
        ));
        assert!(is_route_allowed_impl(
            "/users/settings",
            Some("user2"),
            &routes
        ));
        assert!(!is_route_allowed_impl(
            "/users/profile",
            Some("guest"),
            &routes
        ));
        assert!(!is_route_allowed_impl("/users/profile", None, &routes));
    }

    #[test]
    fn test_nested_routes() {
        let routes = create_test_routes();
        assert!(is_route_allowed_impl(
            "/users/user2/page",
            Some("user2"),
            &routes
        ));
        assert!(!is_route_allowed_impl(
            "/users/user2/page",
            Some("user1"),
            &routes
        ));
        assert!(is_route_allowed_impl(
            "/users/normal/page",
            Some("user1"),
            &routes
        ));
    }

    #[test]
    fn test_api_routes() {
        let routes = create_test_routes();
        assert!(is_route_allowed_impl(
            "/api/v1/users",
            Some("api_user"),
            &routes
        ));
        assert!(is_route_allowed_impl(
            "/api/v1/admin/users",
            Some("admin"),
            &routes
        ));
        assert!(!is_route_allowed_impl(
            "/api/v1/admin/users",
            Some("api_user"),
            &routes
        ));
        assert!(is_route_allowed_impl(
            "/api/v2/users",
            Some("api_user"),
            &routes
        ));
        assert!(is_route_allowed_impl(
            "/api/v2/admin",
            Some("admin"),
            &routes
        ));
    }

    #[test]
    fn test_public_path() {
        let routes = create_test_routes();
        assert!(is_route_allowed_impl(
            "/public/page",
            Some("any_user"),
            &routes
        ));
        assert!(is_route_allowed_impl("/public/page", None, &routes));
    }

    #[test]
    fn test_non_existent_path_with_global_allowed() {
        let routes = create_test_routes();
        assert!(is_route_allowed_impl(
            "/non_existent",
            Some("any_user"),
            &routes
        ));
        assert!(is_route_allowed_impl("/non_existent", None, &routes));
    }

    #[test]
    fn test_non_existent_path_with_global_restricted() {
        let mut routes = HashMap::new();
        routes.insert("/".to_string(), vec!["".to_string()]);
        assert!(!is_route_allowed_impl(
            "/non_existent",
            Some("any_user"),
            &routes
        ));
        assert!(!is_route_allowed_impl("/non_existent", None, &routes));
    }

    #[test]
    fn test_path_order_precedence() {
        let routes = create_test_routes();
        assert!(is_route_allowed_impl(
            "/users/user2/page",
            Some("user2"),
            &routes
        ));
        assert!(!is_route_allowed_impl(
            "/users/user2/page",
            Some("user1"),
            &routes
        ));
        assert!(is_route_allowed_impl(
            "/users/normal/page",
            Some("user1"),
            &routes
        ));
        assert!(is_route_allowed_impl(
            "/users/normal/page",
            Some("user2"),
            &routes
        ));
    }
}
