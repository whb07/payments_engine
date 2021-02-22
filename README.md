# payments_engine

A simple implementation of a payment engine, where an account has the following states:

* valid
* frozen
* disputed

The transition from each stage can only happen in a specified manner:

* valid -> disputed
* disputed -> frozen OR valid

The following actions can be done in each stage:

* valid - withdraw, deposit
* disputed - resolve, chargeback, withdraw, deposit
* frozen: NOTHING


How it works:

It doesn't...yet. Went down the wrong abstraction rabbit hole and down a weird implementation c'est la vie.
