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

        ratings["agree"] = (
            ratings["rating"] == ratings["actual_classification"]
        ) * 100.0

        # Create a duplicate slice for the overall average across both categories
        overall_ratings = ratings.copy()
        overall_ratings["actual_classification"] = "avg_both"

        # Combine them so Seaborn treats 'Average' as just another category hue
        plot_df = pd.concat([ratings, overall_ratings], ignore_index=True)

        sns.set(style="darkgrid")
        plt.figure(figsize=(16, 6))
        sns.barplot(
            data=plot_df,
            x="user_id",
            y="agree",
            errorbar=("ci", 95),
            capsize=0.05,
            hue="actual_classification"
        )

        plt.xlabel("User ID")
        plt.ylabel("Agreement (%)")
        plt.title("Agreement with Classifier by User")
        plt.ylim(0, 100)
        plt.show()
