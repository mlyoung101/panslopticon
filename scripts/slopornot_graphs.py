# Copyright (c) 2026 Mel Young.
#
# This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0. If a copy of the MPL
# was not distributed with this file, You can obtain one at https://mozilla.org/MPL/2.0/.
import matplotlib.pyplot as plt
import pandas as pd
import seaborn as sns
import psycopg
from scipy import stats
from sklearn.metrics import cohen_kappa_score
import numpy as np


if __name__ == "__main__":
    with psycopg.connect("host=lagoon user=postgres dbname=panslopticon") as conn:
        ratings = pd.read_sql_query("SELECT * FROM slopornot_ratings", conn)
        ratings = ratings.drop("id", axis=1)

        kappas = {}

        for user in pd.unique(ratings["user_id"]):
            # exclude myself!
            if user == 1:
                continue
            user_ratings = ratings[ratings.user_id == user]["rating"]
            classifier_ratings = ratings[ratings.user_id == user]["actual_classification"]
            kappas[user] = cohen_kappa_score(user_ratings, classifier_ratings)

        # Calculate the mean and standard error
        k = list(kappas.values())
        mean = np.mean(k)
        se = stats.sem(k)

        # Calculate 95% confidence interval
        confidence = 0.95
        n = len(k)
        dof = n - 1  # degrees of freedom
        confidence_interval = stats.t.interval(confidence, dof, mean, se)

        print(f"Mean: {mean:.2f}")
        print(f"SE: {se:.2f}")
        print(f"95% Confidence Interval: {confidence_interval[0]:.2f} to {confidence_interval[1]:.2f}")

        sns.set_theme(style="darkgrid", font="serif")
        plt.figure(figsize=(16, 6))
        sns.barplot(
            data=kappas,
            errorbar=("ci", 95),
            capsize=0.05,
        )

        plt.xlabel("User ID")
        plt.ylabel("Cohen's kappa")
        plt.title("Agreement with Classifier by User")
        plt.ylim(-1.0, 1.0)
        plt.show()
