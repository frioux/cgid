#!/usr/bin/env plackup

use Data::Dumper::Concise;
sub {
  my $e = shift;
  [200, ['Content-Type', 'text/plain'], [$e->{HTTP_HERP}?Dumper(\%ENV):"foo: $e->{HTTP_FOO}"]]
}
