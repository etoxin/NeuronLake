## MODIFIED Requirements

### Requirement: Select one expert per MVP request
The system SHALL select one expert ID for each routed chat completion request in the MVP before backend preparation and generation begin.

#### Scenario: Request is routed before generation
- **WHEN** a valid chat request is received and a trained router instance is available for the current expert registry
- **THEN** the system predicts a single expert ID and passes that expert ID to the backend generation request

## ADDED Requirements

### Requirement: Keep normal routed chat output clean
The system SHALL keep route selection metadata, confidence values, scores, and debug signals out of normal assistant message content.

#### Scenario: Routed backend response omits route debug text
- **WHEN** a normal chat completion request is routed to an expert before backend generation
- **THEN** the assistant message content contains the backend-generated answer without injected route explanation, confidence, or score text
