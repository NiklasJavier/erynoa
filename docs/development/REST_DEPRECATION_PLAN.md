# REST API Deprecation Plan

**Status**: Planning Phase  
**Created**: 2026-01-25  
**Target Removal**: v2.0.0

## Overview

This document outlines the deprecation plan for REST API endpoints in favor of Connect-RPC/gRPC-Web.

## Current Status

- **Primary API**: Connect-RPC/gRPC-Web (via `/api/v1/connect/`)
- **Legacy API**: REST endpoints (via `/api/v1/`)
- **Status**: REST endpoints are still active but deprecated

## Rationale

1. **Connect-RPC Benefits**:
   - Type-safe with Protobuf schemas
   - Better performance (binary encoding option)
   - Automatic code generation
   - Better error handling
   - Streaming support

2. **Maintenance Burden**:
   - Duplicate handler logic for REST and Connect-RPC
   - Two sets of types to maintain
   - Increased test surface area

## Deprecation Timeline

### Phase 1: Current (v0.1.0 - v0.x.x)
- ✅ Connect-RPC is primary API
- ✅ REST endpoints marked as deprecated
- ✅ Frontend migrated to Connect-RPC
- ✅ REST client removed from frontend
- ⏳ REST endpoints still available for backwards compatibility

### Phase 2: Warning Period (v1.0.0 - v1.x.x)
- Add deprecation warnings to REST endpoint responses
- Document migration guide
- Monitor usage metrics
- Provide support for migration

### Phase 3: Removal (v2.0.0)
- Remove REST endpoint handlers
- Remove REST route definitions
- Remove REST models (if not used elsewhere)
- Update documentation

## Affected Endpoints

### Health Service
- `GET /api/v1/health` → `HealthService.Check`
- `GET /api/v1/ready` → `HealthService.Ready`

### Info Service
- `GET /api/v1/info` → `InfoService.GetInfo`

### User Service
- `GET /api/v1/users` → `UserService.List`
- `GET /api/v1/users/{id}` → `UserService.Get`
- `GET /api/v1/me` → `UserService.GetCurrent`

### Storage Service
- `POST /api/v1/storage/upload` → `StorageService.Upload`
- `GET /api/v1/storage` → `StorageService.ListObjects`
- `DELETE /api/v1/storage/{key}` → `StorageService.Delete`
- `HEAD /api/v1/storage/{key}` → `StorageService.Head`
- `GET /api/v1/storage/presigned/upload` → `StorageService.GetPresignedUploadUrl`
- `GET /api/v1/storage/presigned/download` → `StorageService.GetPresignedDownloadUrl`
- `GET /api/v1/storage/buckets` → `StorageService.ListBuckets`
- `POST /api/v1/storage/buckets` → `StorageService.CreateBucket`
- `DELETE /api/v1/storage/buckets/{name}` → `StorageService.DeleteBucket`

## Migration Guide

### Frontend (Already Complete)
- ✅ Migrated from REST to Connect-RPC
- ✅ Removed REST client code
- ✅ Using Protobuf-generated types

### Backend
- REST handlers remain for backwards compatibility
- New features should only implement Connect-RPC handlers
- REST handlers can be removed in v2.0.0

## Breaking Changes

### v2.0.0
- **Breaking**: All REST endpoints removed
- Clients must use Connect-RPC/gRPC-Web
- No backwards compatibility for REST

## Monitoring

Before removal, monitor:
- REST endpoint usage metrics
- Error rates
- Client versions
- Support requests

## Communication

- Document in CHANGELOG
- Add deprecation notices to API responses
- Update API documentation
- Notify API consumers

## Rollback Plan

If issues arise:
- REST endpoints can be re-enabled quickly
- Keep REST handler code until v2.0.0 is stable
- Monitor for 1-2 releases before final removal

## Notes

- REST endpoints are currently still functional
- No immediate action required
- Plan for removal in next major version (v2.0.0)
- Focus on Connect-RPC for all new development
