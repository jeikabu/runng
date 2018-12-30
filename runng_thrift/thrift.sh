#! /usr/bin/env bash

thrift -gen rs -out tests/common tests/test_service.thrift