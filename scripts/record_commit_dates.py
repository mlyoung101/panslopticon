# Copyright (c) 2026 Mel Young.
#
# This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0. If a copy of the MPL
# was not distributed with this file, You can obtain one at https://mozilla.org/MPL/2.0/.
import psycopg
import time
from tqdm import tqdm
from github import Auth, Github, GithubException
import re
import os
from datetime import datetime, timedelta, UTC
from email.utils import parsedate_tz

REPO_REGEX = re.compile(r"https:\/\/github\.com\/(.*)\/(.*)")


if __name__ == "__main__":
    with psycopg.connect("host=lagoon user=postgres dbname=panslopticon") as conn:
        for db in ("slop", "ham"):
            cursor = conn.cursor().execute(f"""
                SELECT * FROM {db} WHERE NOT dead ORDER BY RANDOM() LIMIT 10000
            """)

            token = Auth.Token(os.environ["GH_PAT"])
            github = Github(auth=token)

            print("Fetching data...")
            for row in cursor.fetchall():
                id = row[0]
                url = row[1]
                print(f"Update {url} in {db}")

                matches = re.match(REPO_REGEX, url)
                try:
                    repo = github.get_repo(f"{matches.group(1)}/{matches.group(2)}")
                    most_recent_commit_date = (
                        repo.get_commits().get_page(0)[0].commit.author.date
                    )
                    now = datetime.now(UTC)
                    conn.execute(
                        "UPDATE slop SET date_last_commit = %s WHERE id = %s;",
                        (most_recent_commit_date, id),
                    )
                    conn.execute(
                        "UPDATE slop SET date_last_seen = %s WHERE id = %s;", (now, id)
                    )

                except GithubException as e:
                    print(f"Failed to fetch repo {url}: {e}")
                    # assume dead
                    conn.execute(
                        "UPDATE slop SET dead = TRUE WHERE id = %s",
                        (id,),
                    )

                conn.commit()
