;(() => {
	var implementors = Object.fromEntries([
		[
			'tower_governor',
			[
				[
					'impl <a class="trait" href="https://doc.rust-lang.org/1.91.1/core/marker/trait.Copy.html" title="trait core::marker::Copy">Copy</a> for <a class="struct" href="tower_governor/key_extractor/struct.GlobalKeyExtractor.html" title="struct tower_governor::key_extractor::GlobalKeyExtractor">GlobalKeyExtractor</a>',
				],
				[
					'impl <a class="trait" href="https://doc.rust-lang.org/1.91.1/core/marker/trait.Copy.html" title="trait core::marker::Copy">Copy</a> for <a class="struct" href="tower_governor/key_extractor/struct.PeerIpKeyExtractor.html" title="struct tower_governor::key_extractor::PeerIpKeyExtractor">PeerIpKeyExtractor</a>',
				],
				[
					'impl <a class="trait" href="https://doc.rust-lang.org/1.91.1/core/marker/trait.Copy.html" title="trait core::marker::Copy">Copy</a> for <a class="struct" href="tower_governor/key_extractor/struct.SmartIpKeyExtractor.html" title="struct tower_governor::key_extractor::SmartIpKeyExtractor">SmartIpKeyExtractor</a>',
				],
			],
		],
	])
	if (window.register_implementors) {
		window.register_implementors(implementors)
	} else {
		window.pending_implementors = implementors
	}
})()
//{"start":57,"fragment_lengths":[998]}
