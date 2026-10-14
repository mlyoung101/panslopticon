# Copyright (c) 2026 Mel Young.
#
# This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0. If a copy of the MPL
# was not distributed with this file, You can obtain one at https://mozilla.org/MPL/2.0/.
import matplotlib.pyplot as plt
import pandas as pd
import seaborn as sns
import psycopg
from scipy.stats import ttest_ind


if __name__ == "__main__":
    with psycopg.connect("host=lagoon user=postgres dbname=panslopticon") as conn:
        # returns an interval
        # https://stackoverflow.com/a/952600/5007892
        slop = pd.read_sql_query("SELECT extract(epoch from (date_last_seen - date_added)) FROM slop WHERE dead", conn)
        ham = pd.read_sql_query("SELECT extract(epoch from (date_last_seen - date_added)) FROM ham WHERE dead", conn)

        result = ttest_ind(a=slop["extract"], b=ham["extract"], equal_var=False).pvalue
        print(f"p < {result:.4f}")
