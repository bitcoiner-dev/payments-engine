Basics
=======

Application should build and read/write data according to the requirements. It should also be properly formatted (cargo fmt).

Completeness
============

I have handled all the cases including deposit, withdrawal, dispute, resolution and chargeback.

Correctness
============

Some unit tests have been written to exercise and verify the different cases work as expected.

Safety and Robustness
=====================

As far as I know, I am not doing anything dangerous. Definitely not using "unsafe" :)

Efficiency
==========

I read transactions from the CSV file one by one which are then dispatched to the tokio async task for processing.

There is always room for making things better. I would have liked to separate dispatching of transactions from the read_csv function.

In a real environment, we would likely use channels to dispatch incoming transactions to the async handle_transaction task. This is because there could be multiple sources from which transactions are sourced. In that case, using an MPSC (milti-producer, single consumer) channel should work well in this case.

Maintainability
===============

Of course, code can always be improved. But I have tried to keep it clean.

Further Improvements
====================

Crates like thiserror could be used to improve error handling. I have done some basic error handling like providing an error message when the input file does not exist. Errors encountered during transaction processing are basically ignored such as when a dispute transaction refers to a non-existant deposit.

Separate read_csv from transaction processing completely.

Write more unit tests.
