extern crate docopt;

extern crate tempdir;
extern crate hyper;
extern crate regex;

use self::tempdir::TempDir;

use self::hyper::Client;
use self::hyper::header::Connection;
use self::hyper::header::ConnectionOption;

use std::fs::File;
use std::io::Read;
use std::process::Command;
use std::process::ExitStatus;
use std::path::Path;

docopt!(pub Args derive Debug, "
Usage:
  worker extract-file <repo> <commit> [--file <arg>] [--help] [--git-only]
  worker --help
  worker --version

Options:
  -f --file <arg>  Files to extract from a git repo [default: rabbitci.json].
  -h --help        Show this screen.
  --git-only       Disable direct github download.
  --version        Get version
");

pub fn extract_file(args: &Args) {
    let repo_info = RepoInfo{repo_url: (*args.arg_repo).to_string(),
                             commit: (*args.arg_commit).to_string()};

    if !args.flag_git_only && repo_info.is_github_repo() {
        let file = fetch_file_from_github(&repo_info, &args.flag_file);

        if file != None {
            println!("{}", file.unwrap());
            return;
        }
    }

    let clone_path = fetch_git_repo(&repo_info);
    let filepath = clone_path.path().join(&args.flag_file);

    let mut body = String::new();
    let file = File::open(filepath).unwrap_or_else(|e| {
        panic!("uh oh! Maybe this file doesn't exist. {:?}", e)
    }).read_to_string(&mut body);

    if file.ok() == None {
        panic!("Could not get file!");
    } else {
        println!("{}", body);
    }
}

pub fn parse() -> Args {
    Args::docopt().decode().unwrap_or_else(|e| e.exit())
}

fn fetch_git_repo(repo_info: &RepoInfo) -> TempDir {
    let z = TempDir::new("temp_git_clone_dir").unwrap();
    let q = z.path().to_str().unwrap().to_string();

    let output = Command::new("git").arg("clone").arg(&repo_info.repo_url).arg(q).arg("--depth=30")
        .output().unwrap_or_else(|e| {
            panic!("Failed to run git clone: {}", e)
        });

    let status = output.status;
    if !status.success() {
        panic!("Git clone failed! {}", String::from_utf8_lossy(&output.stderr));
    }

    let status2 = checkout_commit(&repo_info.commit, z.path());

    if !status2.success() {
        let _ = Command::new("git").arg("fetch").arg("--unshallow").current_dir(z.path())
            .output();
        let status2 = checkout_commit(&repo_info.commit, z.path());

        if !status2.success() {
            panic!("Cannot checkout commit")
        }
    }

    z
}

fn checkout_commit(commit: &String, dir: &Path) -> ExitStatus {
    let output = Command::new("git").arg("checkout").arg(commit).current_dir(dir)
        .output().unwrap_or_else(|e| {
            panic!("It was Steve! {:?}", e)
        });
    output.status
}

fn fetch_file_from_github(repo_info: &RepoInfo, file_name: &str) -> Option<String> {
    let mut client = Client::new();

    let (org, repo) = match repo_info.extract_github_repo_tuple() {
        Some((org, repo)) => (org, repo),
        _ => return None
    };

    let url = format!("https://cdn.rawgit.com/{}/{}/{}/{}",
                      org, repo, repo_info.commit, file_name);

    let mut res = client.get(&*url)
        .header(Connection(vec![ConnectionOption::Close]))
        .send().unwrap();

    if res.status.to_u16() != 200u16 {
        return None
    }

    let mut body = String::new();
    match res.read_to_string(&mut body).ok() {
        Some(_) => Some(body),
        None => None
    }
}

#[derive(Debug)]
struct RepoInfo {
    repo_url: String,
    commit: String
}

impl RepoInfo {
    pub fn is_github_repo(&self) -> bool {
        (&*self.repo_url).contains("github.com")
    }

    pub fn extract_github_repo_tuple(&self) -> Option<(String, String)> {
        if !&self.is_github_repo() { return None; }
        let regex = regex!(r"(?:(?:ssh://)?git@github\.com(?::|/)|(?:https?|git|ssh)://github\.com/)(\S+)(?:\.git)");
        let caps = regex.captures(&self.repo_url).unwrap();
        let org_repo_vec: Vec<_> = caps.at(1).unwrap().split("/").collect();
        Some((org_repo_vec[0].to_string(), org_repo_vec[1].to_string()))
    }
}
