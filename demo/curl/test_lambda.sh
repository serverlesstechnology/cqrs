#!/bin/bash

RANDOM=$$
TEST_ACCT="test-acct-$RANDOM"
echo "Using test account: $TEST_ACCT"

TMP_FILE="_TEMPORARY_PAYLOAD_FILE.json"
TEST_URL="http://localhost:8080/2015-03-31/functions/function/invocations"

trap _remove_tmpfile EXIT
function _remove_tmpfile {
    rm $TMP_FILE
}

function call_lambda() {
    local PAYLOAD=$(echo $1 | sed -e "s/\"/\\\\\\\\\"/g")
    sed -e "s/\$ACCOUNT/$TEST_ACCT/" -e "s/\$PAYLOAD/$PAYLOAD/" lambda_payload.json > $TMP_FILE
    curl -i --location --request POST $TEST_URL --data @$TMP_FILE
    echo
    echo
}

echo "Opening an account"
call_lambda "{\"OpenAccount\": {\"account_id\": \"$TEST_ACCT\"}}"

echo "Depositing money"
call_lambda "{\"DepositMoney\":{\"amount\":1000.0}}"

echo "Withdrawing money"
call_lambda "{\"WithdrawMoney\":{\"atm_id\":\"ATM-N468290\",\"amount\":400.0}}"

echo "Writing a check"
call_lambda "{\"WriteCheck\":{\"check_number\":\"1170\",\"amount\":256.28}}"

echo "Checking account status (calling a query)"
PAYLOAD=""
sed -e "s/\$ACCOUNT/$TEST_ACCT/" -e "s/\$PAYLOAD/$PAYLOAD/" -e "s/\POST/GET/" lambda_payload.json > $TMP_FILE
curl -i --request POST $TEST_URL --data @$TMP_FILE
echo

