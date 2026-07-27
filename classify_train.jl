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

    println("Tokenizing...")
    ham = []
    push!(ham, tokenize(ham_linux_docs))
    for file in ham_db.text
        append!(ham, tokenize(file))
    end

    spam = []
    for file in spam_db.text
        append!(spam, tokenize(file))
    end

    # TODO: zstd?
    serialize("data/spam.dat", spam)
    serialize("data/ham.dat", ham)

    return spam, ham
end

function train()
    if isfile("data/ham.dat") && isfile("data/spam.dat")
        println("Deserialise existing data")
        spam = deserialize("data/spam.dat")
        ham = deserialize("data/ham.dat")
    else
        spam, ham = load_data()
    end

    println("$(length(spam)) spam tokens")
    println("$(length(ham)) ham tokens")

    # split test and train set with Julia's cool new MLDataUtils
    # refs:
    # https://discourse.julialang.org/t/simple-tool-for-train-test-split/473/4
    # https://github.com/JuliaML/MLDataUtils.jl
    train_ham, test_ham = splitobs(ham; at=0.8)
    train_spam, test_spam = splitobs(spam; at=0.8)
end

train()
