# Copyright (c) 2026 Mel Young.
#
# This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0. If a copy of the MPL
# was not distributed with this file, You can obtain one at https://mozilla.org/MPL/2.0/.
using TextAnalysis: serialize
using SQLite
using DataFrames
using Serialization
using TextAnalysis
using ProgressMeter
using MLJ
using MLJText
using MLJBase
using Languages
using ThreadsX
using Caching
using Infiltrator
using Random

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
    # based on:
    # https://juliaai.github.io/MLJ.jl/stable/models/MultinomialNBClassifier_NaiveBayes/#MultinomialNBClassifier_NaiveBayes'
    # https://github.com/JuliaAI/MLJText.jl#tf-idf-transformer

    spam, ham = load_data()
    println("$(length(spam)) spam files")
    println("$(length(ham)) ham files")

    # shuffle the corpus to get our test and train set; MLJ requires it to be this way :/
    labels = vcat(repeat(["ham"], length(ham)), repeat(["spam"], length(spam)))
    corpus = vcat(ham, spam)
    labels_with_contents = Random.shuffle!(collect(zip(corpus, labels)))
    # 80% train, 20% test
    train_ratio = Int64(round(0.8 * length(labels_with_contents)))

    # split apart the labels again
    corpus = [x[1] for x in labels_with_contents]
    labels = [x[2] for x in labels_with_contents]
    # @infiltrate

    println("Tokenising...")
    tokenised = ThreadsX.map(doc -> TextAnalysis.tokenize(Languages.English(), doc), corpus)

    println("Computing features...")
    mach1 = machine(CountTransformer(), tokenised) |> MLJ.fit!

    # matrix of counts
    X = MLJ.transform(mach1, tokenised)
    y = coerce(labels, OrderedFactor)

    println("Training classifier...")
    classifier = MultinomialNBClassifier()
    mach2 = machine(classifier, X, y)
    MLJ.fit!(mach2, rows=1:train_ratio)
    serialize("data/model2.dat", mach2)

    println("Predicting...")
    y_prob = MLJ.predict(mach2, rows=train_ratio+1:length(corpus))
    loss = log_loss(y_prob, y[train_ratio+1:length(corpus)])
    println("Loss: $(loss)")

    @infiltrate
end

train()
