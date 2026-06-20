# neuronguard-expert-routing Specification

## Purpose
TBD - created by archiving change train-neuronguard-router. Update Purpose after archive.
## Requirements
### Requirement: Train router from configured experts
The system SHALL train a NeuronGuard router using the expert IDs, domains, routing hints, examples, and derived routing terms from validated `lake.yaml`.

#### Scenario: Router trains for two experts
- **WHEN** `lake.yaml` defines two experts with domains and routing hints
- **THEN** the system can train a router that maps routing examples to those expert IDs

### Requirement: Extract routing features from chat requests
The system SHALL extract routing features from incoming chat messages, including prompt text, code block language tags, imports, file extensions, framework APIs, package names, and error-message terms where present.

#### Scenario: Framework API becomes routing signal
- **WHEN** a request mentions `createFileRoute` and `loader`
- **THEN** the router feature extraction includes TanStack Router-related routing signals

### Requirement: Persist router artifact with expert-set fingerprint
The system SHALL persist a local router artifact tied to the expert set and routing inputs used for training.

#### Scenario: Router artifact records source fingerprint
- **WHEN** router training completes
- **THEN** the artifact records enough metadata to detect changes to configured expert IDs, hints, examples, and domains

### Requirement: Detect stale router artifacts
The system SHALL detect when a persisted router artifact is stale because relevant `lake.yaml` routing inputs have changed.

#### Scenario: Expert hint change invalidates router
- **WHEN** an expert routing hint changes after a router artifact was trained
- **THEN** the system reports the router artifact as stale or rebuilds it before routing requests

### Requirement: Select one expert per MVP request
The system SHALL select one expert ID for each routed chat completion request in the MVP.

#### Scenario: Request is routed before generation
- **WHEN** a valid chat request is received and a current router artifact is available
- **THEN** the system predicts a single expert ID before backend generation starts

### Requirement: Expose routing confidence or scores
The system SHALL expose routing confidence or score information where practical for route inspection and debugging.

#### Scenario: Route result includes scores
- **WHEN** a route inspection command or debug path requests route details
- **THEN** the result includes the selected expert ID and confidence or score information when available

### Requirement: Provide opt-in route debugging
The system SHALL provide opt-in debugging output that explains why an expert was selected without injecting that output into normal OpenCode chat responses.

#### Scenario: Normal chat response omits route explanation
- **WHEN** OpenCode sends a normal chat completion request
- **THEN** the assistant response is not polluted with routing explanation text

#### Scenario: Debug route output shows contributing signals
- **WHEN** route debugging is enabled for a request or inspection command
- **THEN** the output includes selected expert ID and relevant contributing routing signals where available
