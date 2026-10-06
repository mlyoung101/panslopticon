# Copyright (c) 2026 Mel Young.
#
# This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0. If a copy of the MPL
# was not distributed with this file, You can obtain one at https://mozilla.org/MPL/2.0/.
import matplotlib.pyplot as plt
import pandas as pd
import seaborn as sns
import psycopg


if __name__ == "__main__":
    with psycopg.connect("host=lagoon user=postgres dbname=panslopticon") as conn:
        ratings = pd.read_sql_query("SELECT * FROM slopornot_ratings", conn)
        ratings = ratings.drop("id", axis=1)
        # unique users
        users = sorted(list(set(ratings["user_id"])))
        print(ratings)

        # for user_id in users:
        #     target = ratings[ratings.user_id == user_id]
        #     correct = target[target.rating == target.actual_classification]
        #     agree = (len(correct) / len(target)) * 100.0
        #     print(f"ID: {user_id} Agree: {agree:.2f}%")

        ratings["is_correct"] = (
            ratings["rating"] == ratings["actual_classification"]
        ) * 100.0

        sns.set(style="darkgrid")
        plt.figure(figsize=(10, 6))
        sns.barplot(
            data=ratings,
            x="user_id",
            y="is_correct",
            errorbar=("ci", 95),    # TODO: or std error??
            capsize=0.05,
        )

        plt.xlabel("User ID")
        plt.ylabel("Agreement (%)")
        plt.title("Agreement with Classifier by User")
        plt.ylim(0, 100)
        plt.show()
