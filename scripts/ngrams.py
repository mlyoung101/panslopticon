# Copyright (c) 2026 Mel Young.
#
# This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0. If a copy of the MPL
# was not distributed with this file, You can obtain one at https://mozilla.org/MPL/2.0/.
import numpy as np
import pandas as pd
import rich
from IPython import embed
from sklearn.feature_extraction.text import CountVectorizer, TfidfVectorizer
import psycopg
from markdown_it import MarkdownIt
from mdit_plain.renderer import RendererPlain

parser = MarkdownIt(renderer_cls=RendererPlain)

# based on: https://stackoverflow.com/questions/11763613/python-list-of-ngrams-with-frequencies

LIMIT = 1_000


def load_data() -> pd.DataFrame:
    with psycopg.connect("host=lagoon user=postgres dbname=panslopticon") as conn:
        spam = pd.read_sql_query(
            f"SELECT text FROM full_text WHERE LOWER(file) = 'readme.md' ORDER BY RANDOM() LIMIT {
                LIMIT
            }",
            conn,
        )
        ham = pd.read_sql_query(
            f"SELECT text FROM ham_full_text WHERE LOWER(file) = 'readme.md' ORDER BY RANDOM() LIMIT {
                LIMIT
            }",
            conn,
        )
        conn.close()

        spam["label"] = "spam"
        ham["label"] = "ham"

    return pd.concat([spam, ham], ignore_index=True)

def ngrams(dataset):
    vec = CountVectorizer(ngram_range=(2, 3), stop_words="english", strip_accents="ascii")

    # input to fit_transform() should be an iterable with strings
    ngrams = vec.fit_transform(dataset)

    # needs to happen after fit_transform()
    vocab = vec.vocabulary_

    count_values = ngrams.toarray().sum(axis=0)

    # output n-grams
    for ng_count, ng_text in sorted([(count_values[i],k) for k,i in vocab.items()], reverse=True)[:100]:
        print(ng_count, ng_text)

def train():
    print("Loading data...")
    data: pd.DataFrame = load_data().sample(frac=1)

    slop = [parser.render(x) for x in data[data.label == "spam"]["text"]]
    ham = [parser.render(x) for x in data[data.label == "ham"]["text"]]

    print("\nSlop n-grams:")
    ngrams(slop)

    print("\nHam n-grams:")
    ngrams(ham)


if __name__ == "__main__":
    train()
