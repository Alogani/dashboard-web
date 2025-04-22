use auth::generate_password_hash;
use config::UsersConfig;
use std::{
    io::{self, BufRead, Write},
    path::Path,
};

pub fn manage_users<P: AsRef<Path>>(users_file_path: P) -> Result<(), Box<dyn std::error::Error>> {
    println!("User Management");
    println!("==============");

    // Load existing users
    let mut users_config = match UsersConfig::from_file(&users_file_path) {
        Ok(config) => config,
        Err(e) => {
            println!("Warning: Could not load existing users file: {}", e);
            println!("Creating a new users file.");
            UsersConfig {
                users: std::collections::HashMap::new(),
            }
        }
    };

    let stdin = io::stdin();
    let mut reader = stdin.lock();
    let mut input = String::new();

    loop {
        println!("\nOptions:");
        println!("1. Add/Update user");
        println!("2. List users");
        println!("3. Delete user");
        println!("4. Exit");
        print!("Select an option (1-4): ");
        io::stdout().flush()?;

        input.clear();
        reader.read_line(&mut input)?;

        match input.trim() {
            "1" => add_or_update_user(&mut users_config, &mut reader)?,
            "2" => list_users(&users_config),
            "3" => delete_user(&mut users_config, &mut reader)?,
            "4" => {
                println!("Exiting user management.");
                break;
            }
            _ => println!("Invalid option. Please try again."),
        }
    }

    // Save the updated users configuration
    users_config.to_file(&users_file_path)?;
    println!("Users file saved successfully.");

    Ok(())
}

fn add_or_update_user(
    users_config: &mut UsersConfig,
    reader: &mut impl BufRead,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut input = String::new();

    print!("Enter username: ");
    io::stdout().flush()?;
    input.clear();
    reader.read_line(&mut input)?;
    let username = input.trim().to_string();

    if username.is_empty() {
        println!("Username cannot be empty.");
        return Ok(());
    }

    // Check if user already exists
    let user_exists = users_config.get_password_hash(&username).is_some();
    if user_exists {
        print!(
            "User '{}' already exists. Update password? (y/n): ",
            username
        );
        io::stdout().flush()?;
        input.clear();
        reader.read_line(&mut input)?;

        if !input.trim().eq_ignore_ascii_case("y") {
            println!("Operation cancelled.");
            return Ok(());
        }
    }

    // Get password
    let password = get_password_with_confirmation(reader)?;

    // Generate hash and update user
    match generate_password_hash(&password) {
        Ok(hash) => {
            users_config.users.insert(username.clone(), hash);
            println!(
                "User '{}' {} successfully.",
                username,
                if user_exists { "updated" } else { "added" }
            );
            Ok(())
        }
        Err(e) => {
            println!("Error generating password hash: {}", e);
            Err(Box::new(e))
        }
    }
}

fn get_password_with_confirmation(reader: &mut impl BufRead) -> Result<String, io::Error> {
    let mut input = String::new();

    print!("Enter password: ");
    io::stdout().flush()?;
    input.clear();
    reader.read_line(&mut input)?;
    let password = input.trim().to_string();

    if password.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Password cannot be empty",
        ));
    }

    print!("Confirm password: ");
    io::stdout().flush()?;
    input.clear();
    reader.read_line(&mut input)?;
    let confirm_password = input.trim().to_string();

    if password != confirm_password {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Passwords do not match",
        ));
    }

    Ok(password)
}

fn list_users(users_config: &UsersConfig) {
    println!("\nCurrent Users:");
    println!("-------------");

    if users_config.users.is_empty() {
        println!("No users found.");
        return;
    }

    for username in users_config.users.keys() {
        println!("- {}", username);
    }
}

fn delete_user(
    users_config: &mut UsersConfig,
    reader: &mut impl BufRead,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut input = String::new();

    // List current users first
    list_users(users_config);

    if users_config.users.is_empty() {
        return Ok(());
    }

    print!("\nEnter username to delete: ");
    io::stdout().flush()?;
    input.clear();
    reader.read_line(&mut input)?;
    let username = input.trim().to_string();

    if username.is_empty() {
        println!("Username cannot be empty.");
        return Ok(());
    }

    // Check if user exists
    if !users_config.users.contains_key(&username) {
        println!("User '{}' does not exist.", username);
        return Ok(());
    }

    // Confirm deletion
    print!(
        "Are you sure you want to delete user '{}'? (y/n): ",
        username
    );
    io::stdout().flush()?;
    input.clear();
    reader.read_line(&mut input)?;

    if input.trim().eq_ignore_ascii_case("y") {
        users_config.users.remove(&username);
        println!("User '{}' deleted successfully.", username);
    } else {
        println!("Deletion cancelled.");
    }

    Ok(())
}
