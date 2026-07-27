daemon:
    julia --project --startup-file=no -e 'using DaemonMode; serve()'

train:
    julia --project --startup-file=no -e 'using DaemonMode; runargs()' classify_train.jl
