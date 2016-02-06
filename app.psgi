#!/usr/bin/env plackup

use strict;
use warnings;

use Plack::Builder;

use Data::Dumper::Concise;

builder {
  mount "/echo" => sub {
    my $e = shift;
    my $scheme = 'http';
    [
      200,
      ['Content-Type', 'text/plain'],
      ["$scheme://$e->{SERVER_NAME}:$e->{SERVER_PORT}$e->{SCRIPT_NAME}$e->{PATH_INFO}?$e->{QUERY_STRING}"],
    ],
  };
  mount '/env' => sub {
    my $e = shift;
    my $scheme = 'http';
    [
      200,
      ['Content-Type', 'text/plain'],
      [Dumper(\%ENV)],
    ],
  };
};
