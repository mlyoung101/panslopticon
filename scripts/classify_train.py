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
from sklearn.linear_model import LogisticRegressionCV, LinearRegression
from sklearn.metrics import accuracy_score, classification_report, log_loss
from sklearn.model_selection import train_test_split
from sklearn.naive_bayes import MultinomialNB
from sklearn.neighbors import KNeighborsClassifier
from sklearn.svm import SVC
from sklearn.tree import DecisionTreeClassifier
from sklearn import tree
from sklearn.decomposition import PCA

# based on: https://github.com/nadinejackson1/text-classification-naive-bayes/blob/main/main.ipynb

LIMIT = 12_000

def load_data() -> pd.DataFrame:
    conn = sqlite3.connect("data/panslop.db")
    spam = pd.read_sql_query(
        f"SELECT text FROM full_text ORDER BY RANDOM() LIMIT {LIMIT}", conn
    )
    ham = pd.read_sql_query(
        f"SELECT text FROM ham_full_text ORDER BY RANDOM() LIMIT {LIMIT}", conn
    )
    conn.close()

    spam["label"] = "spam"
    ham["label"] = "ham"

    return pd.concat([spam, ham], ignore_index=True)


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

    with open("data/tfidf_vectorizer.dat", "wb") as f:
        pickle.dump(vectorizer, f)

    with open("data/X_train_vect.dat", "wb") as f:
        pickle.dump(X_train, f)

    with open("data/X_test_vect.dat", "wb") as f:
        pickle.dump(X_test, f)

    # PCA
    # print("Doing PCA")
    # fig = plt.figure(1, figsize=(8, 6))
    # ax = fig.add_subplot(111)
    # X_reduced = PCA(n_components=2).fit_transform(X_train_vect)
    #
    # scatter = ax.scatter(
    #     X_reduced[:, 0],
    #     X_reduced[:, 1],
    #     c=[0 if x == "spam" else 1 for x in y_train],
    #     s=40,
    # )
    #
    # ax.set(
    #     title="First principal components",
    #     xlabel="1st Principal Component",
    #     ylabel="2nd Principal Component",
    # )
    # ax.xaxis.set_ticklabels([])
    # ax.yaxis.set_ticklabels([])
    #
    # # Add a legend
    # legend1 = ax.legend(
    #     scatter.legend_elements()[0],
    #     ["spam", "ham"],
    #     loc="upper right",
    #     title="Classes",
    # )
    # ax.add_artist(legend1)
    #
    # plt.show()

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
        print()

        with open(f"data/classifier_{type(classifier).__name__}.dat", "wb") as f:
            pickle.dump(classifier, f)

        if isinstance(classifier, DecisionTreeClassifier):
            with open("/tmp/tree.dot", "w") as f:
                tree.export_graphviz(
                    classifier,
                    out_file=f,
                    feature_names=vectorizer.get_feature_names_out(),
                    class_names=classifier.classes_,
                    filled=True,
                )

        if isinstance(classifier, MultinomialNB):
            important_features(vectorizer, classifier, 20)


if __name__ == "__main__":
    train()
