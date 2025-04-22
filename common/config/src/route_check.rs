use crate::AppConfig;

impl AppConfig {
    pub fn is_route_allowed(&self, route: &str, username: Option<&str>) -> bool {
        // First, check for exact matches which take highest precedence
        if let Some(allowed_users) = self.allowed_routes.get(route) {
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
        let mut paths: Vec<(&String, &Vec<String>)> = self.allowed_routes.iter().collect();
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
        if let Some(allowed_users) = self.allowed_routes.get("/") {
            if let Some(username) = username {
                return allowed_users.contains(&username.to_string())
                    || allowed_users.contains(&"*".to_string());
            } else {
                return allowed_users.contains(&"*".to_string());
            }
        }

        false
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
