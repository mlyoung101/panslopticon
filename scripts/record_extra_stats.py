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

REPO_REGEX = re.compile(r"https:\/\/github\.com\/(.*)\/(.*)")


if __name__ == "__main__":
    with psycopg.connect("host=lagoon user=postgres dbname=panslopticon") as conn:
        cursor = conn.cursor().execute("""
            SELECT * FROM ham WHERE NOT dead AND (date_created IS NULL OR stars IS NULL)
        """)

        token = Auth.Token(os.environ["GH_PAT"])
        github = Github(auth=token)

        print("Fetching data...")
        for row in tqdm(cursor.fetchall()):
            id = row[0]
            url = row[1]

            matches = re.match(REPO_REGEX, url)
            try:
                repo = github.get_repo(f"{matches.group(1)}/{matches.group(2)}")
                creation_date = repo.created_at
                stars = repo.stargazers_count

                conn.execute(
                    "UPDATE ham SET date_created = %s WHERE id = %s",
                    (creation_date, id),
                    prepare=True,
                )
                conn.execute(
                    "UPDATE ham SET stars = %s WHERE id = %s",
                    (stars, id),
                    prepare=True,
                )
            except GithubException as e:
                print(f"Failed to fetch repo {url}: {e}")
                # assume dead
                conn.execute(
                    "UPDATE ham SET dead = TRUE WHERE id = %s",
                    (id,),
                )

            conn.commit()
            time.sleep(0.3)
