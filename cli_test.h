#pragma once


#include "packetfactory.h"
#include <array>

namespace RoseCommon {
namespace Packet {

class CliTest : public CRosePacket {
    public:
        CliTest();
        CliTest(CRoseReader reader);
        CliTest(CliTest&&) = default;
        CliTest& operator=(CliTest&&) = default;
        ~CliTest() = default;
        
        static constexpr size_t size();
        
        
        
        void set_test(const std::array<int, 42>&);
        void set_test(const int&, size_t index);
        const std::array<int, 42>& get_test() const;
        const int& get_test(size_t index) const;
        
        
        static CliTest create();
        static CliTest create(const uint8_t*);
        static std::unique_ptr<CliTest> allocate(const uint8_t*);
    
    protected:
        virtual void pack(CRoseBasePolicy&) const override;
    
    private:
        std::array<int,42> test;
};

}
}
