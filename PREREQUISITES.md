# PREREQUISITES: HANDLE THE FOLLOWING SUBTASKS IN CHRONOLOGICAL ORDER PRIOR TO VIEWING THE TASK.MD FILE. 
# DELIVERABLE: OUTPUT THE RESULT AND ANY RELEVANT INFORMATION TO A "PREREQUISITE_RESULTS.MD" FILE IN THE PROJECT ROOT.

1. As per the documentation file docs/database/README.md:

    "Caveat (IDs): The current create path in routes creates a Todo with an ID derived from a UUID string parsed into i64, which will panic at runtime. Prefer letting the database assign BIGSERIAL IDs and returning the inserted row. Adjust insert queries accordingly as you evolve the template."


2. As per the documentation file docs/deployment/README.md: 

    Consider adding a separate job for tests (cargo test) before deploy (if you must know the implementation details of the tests prior to completing this step, you may skip this step and proceed to TASK.md)

