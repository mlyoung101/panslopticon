# GENERATED!!!
import argparse
import os
import sys

from github import Auth, Github
from rich import print
from tqdm import tqdm

OUTPUT_PATH = "/tmp/stars.txt"

if __name__ == "__main__":
    parser = argparse.ArgumentParser(
        description="Fetch all starred repos for a GitHub user and write their URLs to /tmp/stars.txt"
    )
    parser.add_argument("username", help="GitHub username to fetch starred repos for")
    args = parser.parse_args()

    token_env = os.environ.get("GH_PAT")
    if not token_env:
        print("[bold red]Error: GH_PAT environment variable is not set[/bold red]")
        sys.exit(1)

    github = Github(auth=Auth.Token(token_env))

    try:
        user = github.get_user(args.username)
        starred = user.get_starred()
        total = starred.totalCount
        print(
            f"Fetching [bold]{total}[/bold] starred repos for [bold]{user.login}[/bold]..."
        )

        repos = []
        for repo in tqdm(starred, total=total, desc="Fetching stars", unit="repo"):
            repos.append(repo)
    except Exception as e:
        print(f"[bold red]Failed to fetch starred repos: {e}[/bold red]")
        sys.exit(1)

    with open(OUTPUT_PATH, "w") as f:
        for repo in repos:
            f.write(repo.html_url + "\n")

    print(f"Wrote [bold green]{len(repos)}[/bold green] repo URLs to {OUTPUT_PATH}")
