# Copyright (c) 2026 Mel Young.
#
# This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0. If a copy of the MPL
# was not distributed with this file, You can obtain one at https://mozilla.org/MPL/2.0/.
import matplotlib.pyplot as plt
import pandas as pd
import seaborn as sns
import psycopg
import statsmodels.formula.api as smf
from patsy import dmatrices

if __name__ == "__main__":
    with psycopg.connect("host=lagoon user=postgres dbname=panslopticon") as conn:
        print("Collecting data...")
        slop = pd.read_sql_query(
            """
            SELECT *,
            EXTRACT(epoch FROM date_last_seen - date_added)/86400 AS lifetime_days,
            date_last_commit >= '2026-01-01 00:00:00.00' AND NOW() - date_last_commit >= make_interval(days => 60) AS abandoned
            FROM slop WHERE stars IS NOT NULL AND date_last_commit IS NOT NULL
            """,
            conn,
        )
        ham = pd.read_sql_query(
            """
            SELECT *,
            EXTRACT(epoch FROM date_last_seen - date_added)/86400 AS lifetime_days,
            date_last_commit >= '2026-01-01 00:00:00.00' AND NOW() - date_last_commit >= make_interval(days => 60) AS abandoned
            FROM ham WHERE stars IS NOT NULL AND date_last_commit IS NOT NULL
            """,
            conn,
        )

        # combine
        slop["is_slop"] = 1
        ham["is_slop"] = 0
        print(slop)

        # necessary for OLS, apparently
        # https://stackoverflow.com/a/27362540/5007892
        slop["abandoned"] = slop["abandoned"].astype(int)
        ham["abandoned"] = ham["abandoned"].astype(int)

        df = pd.concat([slop, ham])
        print(df)

        # compute logit of abandoned
        abandoned_model = smf.logit("abandoned ~ stars + lifetime_days + is_slop", data=df).fit()
        print(abandoned_model.summary())
