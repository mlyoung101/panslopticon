using TextAnalysis: serialize
# Copyright (c) 2026 Mel Young.
#
# This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0. If a copy of the MPL
# was not distributed with this file, You can obtain one at https://mozilla.org/MPL/2.0/.
using SQLite
using DataFrames
using WordTokenizers
using StatsBase
using Serialization
using MLDataUtils
using TextAnalysis

# Trains the classifiers on the spam (AI) and ham (human) datasets

function load_data()
    println("Fetching data from db...")
    db = SQLite.DB("data/panslop.db")
    spam_db = DBInterface.execute(db, "SELECT * FROM full_text") |> DataFrame
    ham_db = DBInterface.execute(db, "SELECT * FROM ham_full_text") |> DataFrame

    ham_linux_docs = read("ham/LINUX_DOCS.md", String)
    println("Loaded.")

    spam = spam_db.text
    ham = ham_db.text
    push!(ham, ham_linux_docs)

    return spam, ham
end

function train()
    spam, ham = load_data()
    println("$(length(spam)) spam files")
    println("$(length(ham)) ham files")

    # split test and train set with Julia's cool new MLDataUtils
    # refs:
    # https://discourse.julialang.org/t/simple-tool-for-train-test-split/473/4
    # https://github.com/JuliaML/MLDataUtils.jl
    train_ham, test_ham = splitobs(ham; at=0.8)
    train_spam, test_spam = splitobs(spam; at=0.8)

    classifier = NaiveBayesClassifier([:spam, :ham])

    println("Training spam...")
    for file in train_spam
        TextAnalysis.fit!(classifier, file, :spam)
    end

    println("Training ham...")
    for file in train_ham
        TextAnalysis.fit!(classifier, file, :ham)
    end

    println("Serializing...")
    serialize("data/classifier.dat", classifier)
    serialize("data/train_ham.dat", train_ham)
    serialize("data/train_spam.dat", train_spam)
    serialize("data/test_ham.dat", test_ham)
    serialize("data/test_spam.dat", test_spam)

    println("Done.")
end

train()
