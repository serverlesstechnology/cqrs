### Event Sourcing

Event sourcing adds to the flexibility of CQRS by relying upon the events as our source of truth. Events are now
historical facts that we can use to calculate the current state in ways that we did not originally intend.
In this way, event sourcing acts similar to a financial ledger.

|   **_Event Name_**   |             **_Event Value_**             | **_Account Balance_** |
|:---------------------|:------------------------------------------|----------------------:|
| _account-opened_     | Account XYZ opened by user John Doe       |                 $0.00 |
| _money-deposited_    | John Doe deposited $500                   |               $500.00 |
| _check-cleared_      | Check #1127 cleared for $27.15            |               $472.85 |
| _cash-withdrawn_     | $100 cash withdrawn from ATM #243         |               $372.85 |

In this example we have four events that have been stored. Consider that a new business requirement is added to track 
money outflow from accounts.
There is no way to do this immediately if the current state is all that is persisted (i.e., the current account balance).

However, because our source of truth is these events rather than a simple balance, it is easy to add the business 
logic after the fact and calculate what we need from the persisted events.

|   **_Event Name_**   |             **_Event Value_**             | **_Account Balance_** | **_Outflow_** |
|:---------------------|:------------------------------------------|----------------------:|--------------:|
| _account-opened_     | Account XYZ opened by user John Doe       |                 $0.00 |         $0.00 |
| _money-deposited_    | John Doe deposited $500                   |               $500.00 |         $0.00 |
| _check-cleared_      | Check #1127 cleared for $27.15            |               $472.85 |        $27.15 |
| _cash-withdrawn_     | $100 cash withdrawn from ATM #243         |               $372.85 |       $127.15 |

In this way, we have used the persisted events to provide new business information that otherwise would not have been 
available. In essence we have turned back time and we can now use new logic to compute states that would
otherwise be impossible to know.
