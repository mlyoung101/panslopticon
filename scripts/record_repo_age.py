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
        cursor = conn.cursor().execute("SELECT * FROM slop WHERE NOT dead AND date_created IS NULL")

        token = Auth.Token(os.environ["GH_PAT"])
        github = Github(auth=token)

        print("Fetching data...")
        for row in tqdm(cursor.fetchall()):
            id = row[0]
            url = row[1]

            matches = re.match(REPO_REGEX, url)
            try:
                creation_date = github.get_repo(f"{matches.group(1)}/{matches.group(2)}").created_at
            except GithubException as e:
                print(f"Failed to fetch repo {url}: {e}")
                continue

            rows_changed = conn.execute("UPDATE slop SET date_created = %s WHERE id = %s", (creation_date, id)).rowcount
            if rows_changed == 0:
                raise Exception("update query probably failed!")
            conn.commit()

            time.sleep(0.3)
