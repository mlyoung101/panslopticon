# Copyright (c) 2026 Mel Young.
#
# This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0. If a copy of the MPL
# was not distributed with this file, You can obtain one at https://mozilla.org/MPL/2.0/.
import os
import re
from datetime import UTC, datetime

import psycopg
from github import Auth, Github, GithubException
from rich import print
from tqdm import tqdm

REPO_REGEX = re.compile(r"https:\/\/github\.com\/(.*)\/(.*)")

# Records most recent commit dates for all repos. This ideally should be run regularly to keep up to date.

if __name__ == "__main__":
    with psycopg.connect("host=lagoon user=postgres dbname=panslopticon") as conn:
        for db in ("ham", "slop"):
            cursor = conn.cursor().execute(f"""
                SELECT * FROM {db} WHERE NOT dead AND date_last_commit IS NULL ORDER BY RANDOM() LIMIT 10000
            """)

            token = Auth.Token(os.environ["GH_PAT"])
            github = Github(auth=token)

            print(f"Fetching data for db {db}...")
            for row in cursor.fetchall():
                id = row[0]
                url = row[1]
                print(f"[green]Update {url} in {db}[/green]")

                matches = re.match(REPO_REGEX, url)
                try:
                    repo = github.get_repo(f"{matches.group(1)}/{matches.group(2)}")
                    most_recent_commit_date = (
                        repo.get_commits().get_page(0)[0].commit.author.date
                    )
                    now = datetime.now(UTC)
                    conn.execute(
                        f"UPDATE {db} SET date_last_commit = %s WHERE id = %s;",
                        (most_recent_commit_date, id),
                    )
                    conn.execute(
                        f"UPDATE {db} SET date_last_seen = %s WHERE id = %s;", (now, id)
                    )

                except Exception as e:
                    print(f"[bold red]Failed to fetch repo {url}: {e}[/bold red]")
                    # assume dead
                    conn.execute(
                        f"UPDATE {db} SET dead = TRUE WHERE id = %s",
                        (id,),
                    )

                conn.commit()
