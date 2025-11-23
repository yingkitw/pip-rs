# pip-main Analysis - Pending Migration Items

**Date**: November 23, 2025  
**Source**: /Users/yingkitw/Downloads/pip-main 2  
**Analysis**: Identified pending work and migration candidates

---

## Overview

The official pip repository contains numerous TODO/FIXME comments indicating pending work and architectural improvements. This document identifies items that could be migrated or implemented in pip-rs.

---

## Pending Items by Category

### 1. Performance & Optimization

#### Dependency Iteration Caching
- **File**: `src/pip/_internal/resolution/resolvelib/candidates.py:229`
- **Issue**: Dependencies are iterated at least twice, should be cached
- **Impact**: Performance improvement for dependency resolution
- **Status**: TODO
- **Priority**: Medium

#### Editable Mode Caching
- **File**: `src/pip/_internal/metadata/base.py:162`
- **Issue**: `editable_project_location` property is costly to compute, needs memoization
- **Impact**: Performance improvement for editable installs
- **Status**: TODO
- **Priority**: Medium

#### Lazy Wheel Optimization
- **File**: `src/pip/_internal/index/collector.py:338`
- **Issue**: PEP 691 style responses not supported in file:// URLs
- **Impact**: Better support for local package indexes
- **Status**: TODO
- **Priority**: Low

---

### 2. Architecture & Design

#### Requirement Preparer Separation
- **File**: `src/pip/_internal/operations/prepare.py:568`
- **Issue**: Separate RequirementPreparer from v1 resolver logic
- **Impact**: Cleaner architecture after v1 resolver removal
- **Status**: TODO
- **Priority**: Medium
- **Note**: Depends on v1 resolver deprecation

#### Candidate Selection Logic
- **File**: `src/pip/_internal/resolution/resolvelib/factory.py:194`
- **Issue**: Check already installed candidate and use if link/editable match
- **Impact**: Better handling of reinstalls
- **Status**: TODO
- **Priority**: Medium

#### Uninstall Logic Improvement
- **File**: `src/pip/_internal/resolution/resolvelib/factory.py:613`
- **Issue**: Determine more cases for uninstallation (especially editable)
- **Impact**: More robust uninstall handling
- **Status**: TODO
- **Priority**: Low

---

### 3. Feature Enhancements

#### Direct URL Support
- **File**: `src/pip/_internal/build_env.py:370`
- **Issue**: Consider direct URL in conflict detection
- **Impact**: Better handling of direct URL dependencies
- **Status**: FIXME
- **Priority**: Medium

#### Environment Marker Evaluation
- **File**: `src/pip/_internal/models/installation_report.py:51`
- **Issue**: Currently uses default environment, should support --python-version and --platform
- **Impact**: Better cross-platform package resolution
- **Status**: TODO
- **Priority**: Medium

#### Egg-Link Project Location
- **File**: `src/pip/_internal/metadata/base.py:172`
- **Issue**: Get project location from second line of egg_link file
- **Impact**: Better editable install support
- **Status**: TODO
- **Priority**: Low
- **Reference**: https://github.com/pypa/pip/issues/10243

#### SVN Entries Handling
- **File**: `src/pip/_internal/vcs/subversion.py:60`
- **Issue**: Should warn if SVN entries file missing
- **Impact**: Better error reporting for SVN sources
- **Status**: FIXME
- **Priority**: Low

---

### 4. File Format & Parsing

#### Requirements File Continuation
- **File**: `src/pip/_internal/req/req_file.py:522`
- **Issue**: Handle space after backslash in continuation lines
- **Impact**: Better requirements file parsing
- **Status**: TODO
- **Priority**: Low

#### Find-Links Tracking
- **File**: `src/pip/_internal/req/req_file.py:255`
- **Issue**: Track source of find_links, support relative paths
- **Impact**: Better requirements file handling
- **Status**: FIXME
- **Priority**: Low

#### Archive Format Detection
- **File**: `src/pip/_internal/utils/unpacking.py:353`
- **Issue**: Handle unknown archive formats and magic signatures
- **Impact**: Better error messages for unsupported formats
- **Status**: FIXME
- **Priority**: Low

---

### 5. Configuration & Locations

#### Virtual Environment Site-Packages
- **File**: `src/pip/_internal/locations/base.py:16`
- **Issue**: Doesn't account for venv linked to global site-packages
- **Impact**: Better venv support
- **Status**: FIXME
- **Priority**: Medium

#### Source Directory Handling
- **File**: `src/pip/_internal/locations/base.py:60`
- **Issue**: Keep src in cwd for now (not a temporary folder)
- **Impact**: Cleaner temporary directory handling
- **Status**: FIXME
- **Priority**: Low

#### Stdlib Packages Definition
- **File**: `src/pip/_internal/metadata/base.py:32`
- **Issue**: Move stdlib_packages definition from utils.compat to metadata
- **Impact**: Better code organization
- **Status**: TODO
- **Priority**: Low

---

### 6. Inspection & Reporting

#### Inspect Command Enhancement
- **File**: `src/pip/_internal/commands/inspect.py:60`
- **Issue**: Add tags and scheme to inspect output
- **Impact**: More comprehensive package information
- **Status**: TODO
- **Priority**: Low

#### Installation Report Environment
- **File**: `src/pip/_internal/models/installation_report.py:51`
- **Issue**: Support environment_override field for --python-version and --platform
- **Impact**: Better cross-platform reporting
- **Status**: TODO
- **Priority**: Medium

---

### 7. Dependency Resolution

#### Installable Directory Test
- **File**: `src/pip/_internal/req/constructors.py:309`
- **Issue**: is_installable_dir test might not be necessary after pyproject.toml changes
- **Impact**: Simplify path handling
- **Status**: TODO
- **Priority**: Low

#### Dataclass Slots Support
- **File**: `src/pip/_internal/models/selection_prefs.py:6`
- **Issue**: Convert to dataclass with slots when Python 3.10+ required
- **Impact**: Memory efficiency improvement
- **Status**: TODO
- **Priority**: Low
- **Blocker**: Python 3.9 support requirement

#### ParsedRequirement Slots
- **File**: `src/pip/_internal/req/req_file.py:103`
- **Issue**: Replace with slots=True when dropping Python 3.9 support
- **Impact**: Memory efficiency improvement
- **Status**: TODO
- **Priority**: Low
- **Blocker**: Python 3.9 support requirement

---

## Summary Statistics

| Category | Count | Priority |
|----------|-------|----------|
| Performance | 2 | Medium |
| Architecture | 3 | Medium |
| Features | 4 | Medium |
| Parsing | 3 | Low |
| Configuration | 3 | Low |
| Inspection | 2 | Medium |
| Resolution | 3 | Low |
| **Total** | **20** | - |

---

## Migration Candidates for pip-rs

### High Priority (Implement First)
1. ✅ Dependency iteration caching
2. ✅ Editable mode caching
3. ✅ Virtual environment site-packages handling
4. ✅ Environment marker evaluation with platform support
5. ✅ Direct URL support in conflict detection

### Medium Priority
1. ✅ Candidate selection logic improvement
2. ✅ Installation report environment override
3. ✅ Archive format detection
4. ✅ Requirements file continuation handling
5. ✅ Find-links tracking

### Low Priority (Nice to Have)
1. ✅ Egg-link project location
2. ✅ SVN entries handling
3. ✅ Dataclass slots optimization
4. ✅ Installable directory test simplification
5. ✅ PEP 691 support for file:// URLs

---

## Architectural Insights

### Resolver Design
- Current pip uses resolvelib with factory pattern
- Candidates are created lazily
- Dependencies need caching for performance
- Already installed candidates should be reused

### Metadata Handling
- Editable mode detection is complex
- Multiple sources: direct_url, egg_link, location
- Caching would improve performance significantly

### Environment Handling
- Currently uses default environment for markers
- Should support platform/version overrides
- Important for cross-platform builds

### Dependency Resolution
- Two-pass iteration of dependencies (inefficient)
- Needs caching mechanism
- Performance bottleneck for large projects

---

## Recommendations for pip-rs

### Implement These First
1. **Caching Layer**: Implement dependency and editable mode caching
2. **Environment Support**: Full environment marker evaluation with overrides
3. **Direct URL Handling**: Proper support for direct URLs in resolution
4. **Performance**: Optimize dependency iteration and resolution

### Architecture Improvements
1. **Cleaner Separation**: Separate resolver from preparer
2. **Better Candidate Selection**: Reuse installed candidates when appropriate
3. **Comprehensive Reporting**: Include tags, scheme, and environment info

### Future Enhancements
1. **PEP 691 Support**: Modern package index format
2. **Better Error Messages**: Improved archive format detection
3. **VCS Improvements**: Better SVN and other VCS support

---

## Files to Review

### Core Resolution
- `src/pip/_internal/resolution/resolvelib/candidates.py`
- `src/pip/_internal/resolution/resolvelib/factory.py`

### Metadata & Locations
- `src/pip/_internal/metadata/base.py`
- `src/pip/_internal/locations/base.py`

### Requirements Handling
- `src/pip/_internal/req/req_file.py`
- `src/pip/_internal/req/constructors.py`

### Operations
- `src/pip/_internal/operations/prepare.py`
- `src/pip/_internal/utils/unpacking.py`

### Commands
- `src/pip/_internal/commands/inspect.py`
- `src/pip/_internal/models/installation_report.py`

---

## Conclusion

The official pip codebase contains 20+ pending items across various categories. Most are performance optimizations or architectural improvements. Key candidates for pip-rs migration include:

1. **Dependency caching** (significant performance gain)
2. **Environment marker evaluation** (cross-platform support)
3. **Direct URL handling** (modern package management)
4. **Better candidate selection** (cleaner resolution logic)

These items represent the next phase of pip-rs development and would bring it closer to feature parity with the official pip while maintaining Rust's performance advantages.

---

**Analysis Complete**: 20 pending items identified, categorized, and prioritized for potential migration to pip-rs.
