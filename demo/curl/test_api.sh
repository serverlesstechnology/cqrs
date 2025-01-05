#!/bin/bash

RANDOM=$$
TEST_ACCT="test-acct-$RANDOM"
TEST_URL="localhost:3030/account/$TEST_ACCT"
echo "Using test account: $TEST_ACCT"
echo "Opening an account"
curl -i --location --request POST $TEST_URL --header 'Content-Type: application/json' --data-raw "{\"OpenAccount\": {\"account_id\": \"$TEST_ACCT\"}}"
echo "Depositing money"
curl -i --location --request POST $TEST_URL --header 'Content-Type: application/json' --data "@DepositMoney.json"
echo "Withdrawing money"
curl -i --location --request POST $TEST_URL --header 'Content-Type: application/json' --data "@WithdrawMoney.json"
echo "Writing a check"
curl -i --location --request POST $TEST_URL --header 'Content-Type: application/json' --data "@WriteCheck.json"
echo "Checking account status (calling a query)"
curl -i --location $TEST_URL
echo
