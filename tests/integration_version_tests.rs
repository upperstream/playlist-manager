use assert_cmd::Command;
use predicates::prelude::*;
use std::path::Path;

#[cfg(test)]
mod tests {
    use super::*;

    // Helper function to get the path to the plm script
    fn get_plm_path() -> String {
        let project_root = Path::new(env!("CARGO_MANIFEST_DIR"));
        let plm_path = project_root.join("bin").join("plm");
        plm_path.to_string_lossy().to_string()
    }

    #[test]
    fn test_version_subcommand() {
        let plm_path = get_plm_path();
        let mut cmd = Command::new(&plm_path);
        let assert = cmd.arg("version").assert();

        assert
            .success()
            .stdout(predicate::str::contains("playlist-manager version"));
    }

    #[test]
    fn test_version_flag_short() {
        let plm_path = get_plm_path();
        let mut cmd = Command::new(&plm_path);
        let assert = cmd.arg("-V").assert();

        assert
            .success()
            .stdout(predicate::str::contains("playlist-manager version"));
    }

    #[test]
    fn test_version_flag_long() {
        let plm_path = get_plm_path();
        let mut cmd = Command::new(&plm_path);
        let assert = cmd.arg("--version").assert();

        assert
            .success()
            .stdout(predicate::str::contains("playlist-manager version"));
    }

    #[test]
    fn test_help_version() {
        let plm_path = get_plm_path();
        let mut cmd = Command::new(&plm_path);
        let assert = cmd.args(["help", "version"]).assert();

        assert
            .success()
            .stdout(predicate::str::contains("Display version information"));
    }

    #[test]
    fn test_version_help_flag_short() {
        let plm_path = get_plm_path();
        let mut cmd = Command::new(&plm_path);
        let assert = cmd.args(["version", "-H"]).assert();

        assert
            .success()
            .stdout(predicate::str::contains("Display version information"));
    }

    #[test]
    fn test_version_help_flag_long() {
        let plm_path = get_plm_path();
        let mut cmd = Command::new(&plm_path);
        let assert = cmd.args(["version", "--help"]).assert();

        assert
            .success()
            .stdout(predicate::str::contains("Display version information"));
    }
}
