## 1. Package Format

- [ ] 1.1 Define the expert package manifest schema and version field.
- [ ] 1.2 Include expert ID, domain, model artifact reference, routing hints, training metadata, compatibility metadata, and version information.
- [ ] 1.3 Add fixtures for packages with embedded artifacts and external artifact references.

## 2. Export

- [ ] 2.1 Implement export from a configured registry expert to a local package folder or archive.
- [ ] 2.2 Preserve training and compatibility metadata during export.
- [ ] 2.3 Support packages that reference model weights without embedding them.
- [ ] 2.4 Add tests for exported manifest content.

## 3. Import

- [ ] 3.1 Validate package manifest shape and required fields before import.
- [ ] 3.2 Validate artifact availability or record warnings for external artifact references.
- [ ] 3.3 Reject duplicate expert IDs unless explicit overwrite or rename behavior is requested.
- [ ] 3.4 Add imported experts to the runtime registry as first-class entries.
- [ ] 3.5 Add tests for valid import, duplicate ID rejection, and metadata preservation.

## 4. Documentation

- [ ] 4.1 Document the local folder/archive package layout.
- [ ] 4.2 Document how large model weight references should be represented.
- [ ] 4.3 Document how imported experts participate in routing and serving.
