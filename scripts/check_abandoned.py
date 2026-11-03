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

# In order to be considered abandoned a repo must:
# a) have at least one commit since [some date] (we don't care about existing abandoned repos)
# b) have not had any commits to any branches in [x] days

# RFC 2822
ABANDONED_LAST_COMMIT = parsedate_tz("Wed, 1 Jan 2025 00:00:00 +0000")

# Days since no commits where we consider them to be abandoned
ABANDONED_CUTOFF_DAYS = timedelta(days=100)

if __name__ == "__main__":
    with psycopg.connect("host=lagoon user=postgres dbname=panslopticon") as conn:
        cursor = conn.cursor().execute("""
            SELECT * FROM slop WHERE NOT dead ORDER BY RANDOM() LIMIT 5000
        """)

        token = Auth.Token(os.environ["GH_PAT"])
        github = Github(auth=token)

        print("Fetching data and finding maybe abandoned repos...")
        for row in tqdm(cursor.fetchall()):
            id = row[0]
            url = row[1]

            matches = re.match(REPO_REGEX, url)
            try:
                repo = github.get_repo(f"{matches.group(1)}/{matches.group(2)}")
                most_recent_commit_date = repo.get_commits().get_page(0)[0].commit.author.date
                now = datetime.now(UTC)

                if now - most_recent_commit_date >= ABANDONED_CUTOFF_DAYS:
                    print(f"Repo {url} maybe abandoned")

            except GithubException as e:
                print(f"Failed to fetch repo {url}: {e}")
                # assume dead
                conn.execute(
                    "UPDATE slop SET dead = TRUE WHERE id = %s",
                    (id,),
                )

            conn.commit()
