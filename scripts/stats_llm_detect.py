
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
from zippy import EnsembledZippy, LzmaLlmDetector

if __name__ == "__main__":
    with psycopg.connect("host=lagoon user=postgres dbname=panslopticon") as conn:
        print("Collecting data...")
        slop = pd.read_sql_query(
            """
            SELECT * FROM full_text WHERE file = 'README.md' ORDER BY RANDOM() LIMIT 20
            """,
            conn,
        )
        ham = pd.read_sql_query(
            """
            SELECT * FROM ham_full_text WHERE file = 'README.md' ORDER BY RANDOM() LIMIT 20
            """,
            conn,
        )

        # combine
        slop["is_slop"] = 1
        ham["is_slop"] = 0

        detector = EnsembledZippy()

        slop["zippy_score"] = slop.apply(lambda row: detector.run_on_text_chunked(row["text"]), axis=1)
        ham["zippy_score"] = ham.apply(lambda row: detector.run_on_text_chunked(row["text"]), axis=1)

        print("Slop:")
        print(slop)
        print("Ham:")
        print(ham)

        #
        # df = pd.concat([slop, ham])
        # print(df)
        #
        # # compute OLS of lifetime
        # model = smf.ols("lifetime_days ~ stars + score", data=df).fit()
        # print(model.summary())
