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

if __name__ == "__main__":
    with psycopg.connect("host=lagoon user=postgres dbname=panslopticon") as conn:
        slop = pd.read_sql_query(
            "SELECT * FROM slop",
            conn,
        )
        ham = pd.read_sql_query(
            "SELECT * FROM ham",
            conn,
        )
