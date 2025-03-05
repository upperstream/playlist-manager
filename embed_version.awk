#!/usr/bin/awk -f
BEGIN {
	if (ARGC < 2) {
		print_usage();
		exit 1;
	}
	if (ARGV[1] == "--help" || ARGV[1] == "-H" || ARGV[1] == "-h" || ARGV[1] == "-?" || ARGV[1] == "-help" || ARGV[1] == "help") {
		print_usage();
		exit 0;
	}
	extracted_version = extract_version(ARGV[1]);
	if (extracted_version == "") {
		print "Error: Unable to extract version from " ARGV[1];
		exit 1;
	}
	delete ARGV[1];
}

function print_usage() {
	printf("Usage: awk -f embed_version.awk <toml_file> [ <script> ... ] [ > <output> ]\n");
	printf("  where <toml_file> is a Cargo.toml file to extract version number from,\n");
	printf("        <script> is a shell script file to embed the version number into, and\n");
	printf("        <output> is the result shell script file, or printed to standard output stream\n");
}

function extract_version(file) {
    command = sprintf("grep \"^version\" %s | head -n1 | sed 's/^version = \"\\(.*\\)\"/\\1/'", file);
    if (command | getline _tool_version) {
		close(command);
	} else {
		print "Error: Unable to extract version from " file;
		exit 1;
	}
    close(command)
	return _tool_version
}

{
    gsub("@VERSION@", extracted_version);
    print;
}
