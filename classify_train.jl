using BytePairEncoding: TextEncodeBase
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
using BytePairEncoding
using TextEncodeBase

# Trains the classifiers on the spam (AI) and ham (human) datasets

function load_data()
    println("Fetching data from db...")
    db = SQLite.DB("data/panslop.db")
    spam_db = DBInterface.execute(db, "SELECT * FROM full_text ORDER BY RANDOM() LIMIT 1000") |> DataFrame
    ham_db = DBInterface.execute(db, "SELECT * FROM ham_full_text ORDER BY RANDOM() LIMIT 1000") |> DataFrame

    # ham_linux_docs = read("ham/LINUX_DOCS.md", String)
    # println("Loaded.")

    spam = spam_db.text
    ham = ham_db.text
    # push!(ham, ham_linux_docs)

    return spam, ham
end

# Fine! I'll do it myself
function fit!(c::NaiveBayesClassifier, sd::AbstractDocument, class)
    fs = frequencies(tokens(sd))
    for k in keys(fs)
        k in c.dict || extend!(c, k)
    end
    fit!(c, features(fs, c.dict), class)
end

function train()
    spam, ham = load_data()
    println("$(length(spam)) spam files")
    println("$(length(ham)) ham files")

    println("Loading tokenizer model...")
    encoder = BytePairEncoding.load_tiktoken_encoder("cl100k_base")

    # split test and train set with Julia's cool new MLDataUtils
    # refs:
    # https://discourse.julialang.org/t/simple-tool-for-train-test-split/473/4
    # https://github.com/JuliaML/MLDataUtils.jl
    train_ham, test_ham = splitobs(ham; at=0.8)
    train_spam, test_spam = splitobs(spam; at=0.8)

    println("Export training dataset...")
    serialize("data/train_ham.dat", train_ham)
    serialize("data/train_spam.dat", train_spam)
    serialize("data/test_ham.dat", test_ham)
    serialize("data/test_spam.dat", test_spam)

    println("Generating classes...")
    classes = [TextEncodeBase.lookup(encoder.vocab, x) for x in 1:length(encoder.vocab)]
    classifier = NaiveBayesClassifier(classes, [:spam, :ham])

    println("Training spam...")
    @showprogress for file in train_spam
        tokens = encoder.encode(file)
        words = [TextEncodeBase.lookup(encoder.vocab, x) for x in tokens]
        TextAnalysis.fit!(classifier, words, :spam)
    end

    serialize("data/classifier.dat", classifier)

    println("Training ham...")
    @showprogress for file in train_ham
        TextAnalysis.fit!(classifier, file, :ham)
    end

    println("Serializing...")
    serialize("data/classifier.dat", classifier)

    println("Done.")
end

train()
