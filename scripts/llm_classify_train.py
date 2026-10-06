# Copyright (c) 2026 Mel Young.
#
# This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0. If a copy of the MPL
# was not distributed with this file, You can obtain one at https://mozilla.org/MPL/2.0/.
import pickle
import matplotlib.pyplot as plt

import numpy as np
import pandas as pd
import rich
from IPython import embed
from sklearn.ensemble import AdaBoostClassifier, RandomForestClassifier
from sklearn.feature_extraction.text import CountVectorizer, TfidfVectorizer
from sklearn.linear_model import LogisticRegressionCV, LinearRegression
from sklearn.metrics import accuracy_score, classification_report, log_loss
from sklearn.model_selection import train_test_split
from sklearn.naive_bayes import MultinomialNB
from sklearn.neighbors import KNeighborsClassifier
from sklearn.svm import SVC
from sklearn.tree import DecisionTreeClassifier
from sklearn import tree
from sklearn.decomposition import PCA
import psycopg

# based on: https://github.com/nadinejackson1/text-classification-naive-bayes/blob/main/main.ipynb

LIMIT = 6_000


def load_data() -> pd.DataFrame:
    with psycopg.connect("host=lagoon user=postgres dbname=panslopticon") as conn:
        claude = pd.read_sql_query(
            f"SELECT * FROM full_text INNER JOIN agents ON agents.slop_id = full_text.slop_id WHERE agent = 'claude' ORDER BY RANDOM() LIMIT {
                LIMIT
            }",
            conn,
        )
        gemini = pd.read_sql_query(
            f"SELECT * FROM full_text INNER JOIN agents ON agents.slop_id = full_text.slop_id WHERE agent = 'gemini' ORDER BY RANDOM() LIMIT {
                LIMIT
            }",
            conn,
        )
        codex = pd.read_sql_query(
            f"SELECT * FROM full_text INNER JOIN agents ON agents.slop_id = full_text.slop_id WHERE agent = 'codex' ORDER BY RANDOM() LIMIT {
                LIMIT
            }",
            conn,
        )
        conn.close()

        claude["label"] = "claude"
        gemini["label"] = "gemini"
        codex["label"] = "codex"

    return pd.concat([claude, gemini, codex], ignore_index=True)


# https://stackoverflow.com/a/50810751/5007892
def important_features(vectorizer, classifier, n=20):
    class_labels = classifier.classes_
    feature_names = vectorizer.get_feature_names_out()

    topn_class1 = sorted(
        zip(classifier.feature_count_[0], feature_names), reverse=True
    )[:n]
    topn_class2 = sorted(
        zip(classifier.feature_count_[1], feature_names), reverse=True
    )[:n]

    print("Top ham features")

    for coef, feat in topn_class1:
        print(f"{class_labels[0]} {coef:.4f} {feat}")

    print("-----------------------------------------")
    print("Top spam features")

    for coef, feat in topn_class2:
        print(f"{class_labels[1]} {coef:.4f} {feat}")


# https://gist.github.com/Lorenzoantonelli/40454798ae53386a1d5b9c8bb60664d5


def report_to_latex(report):
    if report[0] == "\n":
        report = report[1:]
    if report[-1] == "\n":
        report = report[:-1]

    lines = report.split("\n")

    header = [
        "\\begin{table}",
        "\\caption{Latex Table from Classification Report}",
        "\\label{table:classification:report}",
        "\\centering",
        "\\begin{tabular}{c c c c r}",
        "& Precision & Recall & F-score & Support",
        "\\\\",
    ]

    body = []
    for line in lines[2:-4]:
        row = line.split()
        if len(row) == 5:
            body.append(" & ".join(row) + "\\\\")

    body.append("\\\\")

    footer = []
    for line in lines[-3:]:
        row = line.split()
        if len(row) == 3:
            footer.append("{} & & & {} & {}\\\\".format(*row))
        elif len(row) == 6:
            footer.append("{} {} & {} & {} & {} & {}\\\\".format(*row))

    footer.extend(["\\end{tabular}", "\\end{table}"])

    latex_table = "\n".join(header + body + footer)

    return latex_table


def train():
    print("Loading data...")
    data: pd.DataFrame = load_data().sample(frac=1)
    print(data)

    X = data["text"]
    y = data["label"]
    X_train, X_test, y_train, y_test = train_test_split(X, y, test_size=0.2)

    print("Vectorising...")
    vectorizer = TfidfVectorizer(stop_words="english", max_features=10000)

    X_train_vect = vectorizer.fit_transform(X_train)
    X_test_vect = vectorizer.transform(X_test)

    # based on https://scikit-learn.org/stable/auto_examples/classification/plot_classifier_comparison.html
    classifiers = {
        "Naive Bayes": MultinomialNB(),
        "Decision Tree": DecisionTreeClassifier(max_depth=5),
        "KNN": KNeighborsClassifier(5),
        # "Linear SVM": SVC(kernel="linear", C=0.025),
        "Random Forest": RandomForestClassifier(max_depth=5, n_estimators=15),
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

        rich.print(f"[bold red]Classification report for {name}:[/bold red]\n{report}")
        print(report_to_latex(report))
        print()

        if isinstance(classifier, MultinomialNB):
            important_features(vectorizer, classifier, 20)


if __name__ == "__main__":
    train()
