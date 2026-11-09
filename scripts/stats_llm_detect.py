# Copyright (c) 2026 Mel Young.
#
# This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0. If a copy of the MPL
# was not distributed with this file, You can obtain one at https://mozilla.org/MPL/2.0/.
from typing import List, Tuple
import numpy as np
import matplotlib.pyplot as plt
import pandas as pd
import seaborn as sns
import psycopg
import statsmodels.formula.api as smf
from nltk.corpus import stopwords
import nltk
import string
from scipy.stats import ttest_ind
from rich import print
import swifter


# these two from
# https://github.com/shivamkapoor172002/Language-Model-Text-Analysis-with-Burstiness-and-Perplexity/blob/main/app.py
def preprocess_text(text):
    tokens = nltk.word_tokenize(text.lower())
    stop_words = set(stopwords.words("english"))
    tokens = [
        token
        for token in tokens
        if token not in stop_words and token not in string.punctuation
    ]
    return tokens


def calculate_burstiness(text):
    tokens = preprocess_text(text)
    word_freq = nltk.FreqDist(tokens)

    avg_freq = sum(word_freq.values()) / len(word_freq)
    variance = sum((freq - avg_freq) ** 2 for freq in word_freq.values()) / len(
        word_freq
    )

    burstiness_score = variance / (avg_freq**2)
    return burstiness_score


if __name__ == "__main__":
    with psycopg.connect("host=lagoon user=postgres dbname=panslopticon") as conn:
        print("Collecting data...")
        slop = pd.read_sql_query(
            """
            SELECT * FROM full_text WHERE LOWER(file) = 'readme.md' ORDER BY RANDOM() LIMIT 6000
            """,
            conn,
        )
        ham = pd.read_sql_query(
            """
            SELECT * FROM ham_full_text WHERE LOWER(file) = 'readme.md' ORDER BY RANDOM() LIMIT 6000
            """,
            conn,
        )

        slop["burstiness"] = slop.swifter.apply(
            lambda row: calculate_burstiness(row["text"]), axis=1
        )
        ham["burstiness"] = ham.swifter.apply(
            lambda row: calculate_burstiness(row["text"]), axis=1
        )

        print("Slop:")
        print(slop)
        print("Ham:")
        print(ham)

        result = ttest_ind(
            a=slop["burstiness"], b=ham["burstiness"], equal_var=False
        ).pvalue
        print(f"burstiness p < {result:.16f}")

        print(f"Slop burstiness mean: {slop['burstiness'].mean()}")
        print(f"Ham burstiness mean: {ham['burstiness'].mean()}")
