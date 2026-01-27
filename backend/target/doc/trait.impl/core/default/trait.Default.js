;(() => {
	var implementors = Object.fromEntries([
		[
			'tower_governor',
			[
				[
					'impl <a class="trait" href="https://doc.rust-lang.org/1.91.1/core/default/trait.Default.html" title="trait core::default::Default">Default</a> for <a class="struct" href="tower_governor/governor/struct.GovernorConfig.html" title="struct tower_governor::governor::GovernorConfig">GovernorConfig</a>&lt;<a class="struct" href="tower_governor/key_extractor/struct.PeerIpKeyExtractor.html" title="struct tower_governor::key_extractor::PeerIpKeyExtractor">PeerIpKeyExtractor</a>, NoOpMiddleware&gt;',
				],
				[
					'impl <a class="trait" href="https://doc.rust-lang.org/1.91.1/core/default/trait.Default.html" title="trait core::default::Default">Default</a> for <a class="struct" href="tower_governor/governor/struct.GovernorConfigBuilder.html" title="struct tower_governor::governor::GovernorConfigBuilder">GovernorConfigBuilder</a>&lt;<a class="struct" href="tower_governor/key_extractor/struct.PeerIpKeyExtractor.html" title="struct tower_governor::key_extractor::PeerIpKeyExtractor">PeerIpKeyExtractor</a>, NoOpMiddleware&gt;',
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
//{"start":57,"fragment_lengths":[1073]}
