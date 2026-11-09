# Copyright (c) 2026 Mel Young.
#
# This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0. If a copy of the MPL
# was not distributed with this file, You can obtain one at https://mozilla.org/MPL/2.0/.
import os
import re
import time

import psycopg
from dateutil import parser
from github import Auth, Github, GithubException
from rich import print
from tqdm import tqdm

REPO_REGEX = re.compile(r"https:\/\/github\.com\/(.*)\/(.*)")
DATE_CUTOFF = parser.parse("Wed, 1 Jan 2025 00:00:00 +0000")

# Records date created and stars (and maybe other stats in future). This is a fixup script so it's not
# intended to be run regularly.

if __name__ == "__main__":
    with psycopg.connect("host=lagoon user=postgres dbname=panslopticon") as conn:
        cursor = conn.cursor().execute("""
            SELECT * FROM slop WHERE NOT dead AND (date_created IS NULL OR stars IS NULL)
        """)

        token = Auth.Token(os.environ["GH_PAT"])
        github = Github(auth=token)

        print("Fetching data...")
        for row in cursor.fetchall():
            id = row[0]
            url = row[1]

            matches = re.match(REPO_REGEX, url)
            try:
                repo = github.get_repo(f"{matches.group(1)}/{matches.group(2)}")
                creation_date = repo.created_at
                stars = repo.stargazers_count

                if creation_date < DATE_CUTOFF:
                    print(
                        f"[bold red]Repo {url} created before cutoff, removing[/bold red]"
                    )
                    conn.execute("DELETE FROM slop WHERE id = %s;", id)
                else:
                    conn.execute(
                        "UPDATE slop SET date_created = %s WHERE id = %s",
                        (creation_date, id),
                        prepare=True,
                    )
                    conn.execute(
                        "UPDATE slop SET stars = %s WHERE id = %s",
                        (stars, id),
                        prepare=True,
                    )
                    print(
                        f"[green]Repo {url} - created on {creation_date} - stars {stars}[/green]"
                    )
            except Exception as e:
                print(f"[bold red]Failed to fetch repo {url}: {e}[/bold red]")
                # assume dead
                conn.execute(
                    "UPDATE slop SET dead = TRUE WHERE id = %s",
                    (id,),
                )

            conn.commit()
