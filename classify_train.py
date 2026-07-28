# Copyright (c) 2026 Mel Young.
#
# This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0. If a copy of the MPL
# was not distributed with this file, You can obtain one at https://mozilla.org/MPL/2.0/.
from sklearn.feature_extraction.text import CountVectorizer, TfidfVectorizer
from sklearn.naive_bayes import MultinomialNB
from sklearn.metrics import accuracy_score, log_loss
from sklearn.model_selection import train_test_split
from sklearn.metrics import classification_report
import pandas as pd
import pickle
import sqlite3

# based on: https://github.com/nadinejackson1/text-classification-naive-bayes/blob/main/main.ipynb


def load_data() -> pd.DataFrame:
    conn = sqlite3.connect("data/panslop.db")
    spam = pd.read_sql_query("SELECT text FROM full_text ORDER BY RANDOM() LIMIT 6000", conn)
    ham = pd.read_sql_query("SELECT text FROM ham_full_text ORDER BY RANDOM() LIMIT 6000", conn)
    conn.close()

    spam["label"] = "spam"
    ham["label"] = "ham"

    return pd.concat([spam, ham], ignore_index=True)


def train():
    print("Loading data...")
    data: pd.DataFrame = load_data().sample(frac=1)
    print(data)

    X = data["text"]
    y = data["label"]
    X_train, X_test, y_train, y_test = train_test_split(
        X, y, test_size=0.2, random_state=42
    )

    print("Vectorising...")
    vectorizer = TfidfVectorizer(stop_words="english", max_features=10000)
    X_train_vect = vectorizer.fit_transform(X_train)
    X_test_vect = vectorizer.transform(X_test)

    print("Fitting classifier...")
    classifier = MultinomialNB()
    classifier.fit(X_train_vect, y_train)
    with open("data/model_py.dat", "wb") as f:
        pickle.dump(classifier, f)

    print("Predicting...")
    y_pred = classifier.predict(X_test_vect)

    print(classification_report(y_test, y_pred, target_names=["spam", "ham"]))


if __name__ == "__main__":
    train()
