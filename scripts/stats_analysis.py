# Copyright (c) 2026 Mel Young.
#
# This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0. If a copy of the MPL
# was not distributed with this file, You can obtain one at https://mozilla.org/MPL/2.0/.
import matplotlib.pyplot as plt
import pandas as pd
import seaborn as sns
import psycopg
import statsmodels.api as sm
from patsy import dmatrices
from scipy.stats import f_oneway

if __name__ == "__main__":
    with psycopg.connect("host=lagoon user=postgres dbname=panslopticon") as conn:
        print("Collecting data...")
        slop = pd.read_sql_query(
            "SELECT * FROM slop",
            conn,
        )
        # ham = pd.read_sql_query(
        #     "SELECT * FROM ham",
        #     conn,
        # )

        groups = [group["score"].values for name, group in slop.groupby("origin_src")]

        print(slop)
        print(groups)

        stat, pvalue = f_oneway(*groups)
        print(f"ANOVA F: {stat:.2f}, p = {pvalue:.4e}")
