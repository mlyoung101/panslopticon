# Copyright (c) 2026 Mel Young.
#
# This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0. If a copy of the MPL
# was not distributed with this file, You can obtain one at https://mozilla.org/MPL/2.0/.
import pickle
import sqlite3
import matplotlib.pyplot as plt

import numpy as np
import pandas as pd
import rich
from IPython import embed
from sklearn.ensemble import AdaBoostClassifier, RandomForestClassifier
from sklearn.feature_extraction.text import CountVectorizer, TfidfVectorizer
from sklearn.linear_model import LogisticRegressionCV
from sklearn.metrics import accuracy_score, classification_report, log_loss
from sklearn.model_selection import train_test_split
from sklearn.naive_bayes import MultinomialNB
from sklearn.neighbors import KNeighborsClassifier
from sklearn.svm import SVC
from sklearn.tree import DecisionTreeClassifier
from sklearn import tree

# based on: https://github.com/nadinejackson1/text-classification-naive-bayes/blob/main/main.ipynb


def load_data() -> pd.DataFrame:
    conn = sqlite3.connect("data/panslop.db")
    spam = pd.read_sql_query(
        "SELECT text FROM full_text ORDER BY RANDOM() LIMIT 6000", conn
    )
    ham = pd.read_sql_query(
        "SELECT text FROM ham_full_text ORDER BY RANDOM() LIMIT 6000", conn
    )
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
    vectorizer = CountVectorizer(stop_words="english", max_features=10000)
    X_train_vect = vectorizer.fit_transform(X_train)
    X_test_vect = vectorizer.transform(X_test)

    # based on https://scikit-learn.org/stable/auto_examples/classification/plot_classifier_comparison.html
    classifiers = {
        "Decision Tree": DecisionTreeClassifier(max_depth=5),
        "Naive Bayes": MultinomialNB(),
        "KNN": KNeighborsClassifier(5),
        "Linear SVM": SVC(kernel="linear", C=0.025, random_state=42),
        "Random Forest": RandomForestClassifier(
            max_depth=5, n_estimators=10, max_features=5, random_state=42
        ),
        # "Logistic Regression": LogisticRegressionCV(
        #     Cs=np.logspace(-6, 6, 101),
        #     cv=10,
        #     l1_ratios=(0,),
        #     scoring="neg_log_loss",
        #     max_iter=1_000,
        #     use_legacy_attributes=False,
        # ),
    }

    for name, classifier in classifiers.items():
        print(f"Fitting classifier {name}...")
        classifier.fit(X_train_vect, y_train)

        print("Predicting...")
        y_pred = classifier.predict(X_test_vect)
        report = classification_report(y_test, y_pred)

        rich.print(
            f"[bold red]Classification report for {name}:[/bold red]\n[lime]{
                report
            }[/lime]"
        )
        print()

        if isinstance(classifier, DecisionTreeClassifier):
            tree.plot_tree(classifier, class_names=True)
            plt.show()

    # print("SPAM most salient:")
    # spam_class_prob_sorted = classifier.feature_log_prob_[0, :].argsort()[::-1]
    # print(np.take(vectorizer.get_feature_names_out(), spam_class_prob_sorted[:10]))
    #
    # print("HAM most salient:")
    # ham_class_prob_sorted = classifier.feature_log_prob_[1, :].argsort()[::-1]
    # print(np.take(vectorizer.get_feature_names_out(), ham_class_prob_sorted[:10]))


if __name__ == "__main__":
    train()
