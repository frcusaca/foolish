#!/bin/sh
#
rm -rf ~/.m2/repository/org/foolish
echo "Compiling and running test"
ofn=`mktemp /tmp/.approval.tmp.XXXXXXXXXXXXXXXXXXXXXXX`
mvn clean verify -am -fae -pl foolish-core-java -T $(($(nproc) * 2)) -Dparallel=classesAndMethods -DthreadCount=$(($(nproc) * 4)) 2>&1 | tee $ofn 2>&1
echo "Now editing differences."
for file in $(./approval_extract_failures_from_test.sh $ofn); do
	echo "Checking out this approval file $file"
	sleep 0.2s
	./approval_diff.sh $file
done
#rm $ofn
