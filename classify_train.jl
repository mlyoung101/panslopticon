# Copyright (c) 2026 Mel Young.
#
# This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0. If a copy of the MPL
# was not distributed with this file, You can obtain one at https://mozilla.org/MPL/2.0/.
using TextAnalysis: serialize
using SQLite
using DataFrames
using WordTokenizers
using StatsBase
using Serialization
using MLDataUtils
using TextAnalysis
using ProgressMeter
using MLJ
using MLJText

CountTransformer = @load CountTransformer pkg=MLJText
MultinomialNBClassifier = @load MultinomialNBClassifier pkg=NaiveBayes

# Trains the classifiers on the spam (AI) and ham (human) datasets

function load_data()
    println("Fetching data from db...")
    db = SQLite.DB("data/panslop.db")
    spam_db = DBInterface.execute(db, "SELECT * FROM full_text ORDER BY RANDOM() LIMIT 2000") |> DataFrame
    ham_db = DBInterface.execute(db, "SELECT * FROM ham_full_text ORDER BY RANDOM() LIMIT 2000") |> DataFrame

    # ham_linux_docs = read("ham/LINUX_DOCS.md", String)
    # println("Loaded.")

    spam = spam_db.text
    ham = ham_db.text
    # push!(ham, ham_linux_docs)

    return spam, ham
end

function train()
    spam, ham = load_data()
    corpus = vcat(spam, ham)
    println("$(length(spam)) spam files")
    println("$(length(ham)) ham files")

    # split test and train set with Julia's cool new MLDataUtils
    # refs:
    # https://discourse.julialang.org/t/simple-tool-for-train-test-split/473/4
    # https://github.com/JuliaML/MLDataUtils.jl
    train_ham, test_ham = splitobs(ham; at=0.8)
    train_spam, test_spam = splitobs(spam; at=0.8)

    # based on:
    # https://juliaai.github.io/MLJ.jl/stable/models/MultinomialNBClassifier_NaiveBayes/#MultinomialNBClassifier_NaiveBayes'
    # https://github.com/JuliaAI/MLJText.jl#tf-idf-transformer
    tfidf_transformer = TfidfTransformer()
    mach = machine(tfidf_transformer, TextAnalysis.tokenize.(corpus))
    MLJ.fit!(mach)

    println("Done.")
end

# train()
